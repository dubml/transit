use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, ClapArgs)]
pub struct StealthArgs {
    /// Path to configuration file.
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Path to configuration file (alias for --file).
    #[arg(short = 'c', long = "config", value_name = "CONFIG")]
    pub config: Option<PathBuf>,

    /// Write transit output (stdout/stderr) to file (defaults to silencing output).
    #[arg(long = "output", value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Command and arguments to run after transit is ready.
    #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<OsString>,
}

pub fn execute(args: StealthArgs) -> Result<()> {
    if args.command.is_empty() {
        bail!("stealth mode requires a command to run after '--'");
    }

    if let Some(file) = args.file.as_ref().or(args.config.as_ref()) {
        if file == Path::new("-") {
            bail!(
                "stealth mode does not support reading configuration from standard input ('-f -') \
                 because standard input is inherited by the target command"
            );
        }
    }

    #[cfg(unix)]
    {
        execute_unix(args)
    }

    #[cfg(not(unix))]
    {
        bail!("stealth mode is only supported on Unix targets");
    }
}

#[cfg(unix)]
fn execute_unix(args: StealthArgs) -> Result<()> {
    use std::os::fd::AsRawFd;
    use std::os::unix::process::CommandExt;

    // Block SIGINT & SIGTERM in main thread before spawning so that
    // any spawned threads inherit the mask and signals can be handled synchronously via sigwait.
    let old_sigmask = block_termination_signals()?;

    let (read_fd, write_fd) = create_nonblocking_pipe()?;

    let mut cmd = Command::new(std::env::current_exe().context("failed to locate current executable")?);

    if let Some(file) = args.file.as_ref().or(args.config.as_ref()) {
        cmd.arg("-f").arg(file);
    }

    cmd.env("TRANSIT_READY_FD", "3");

    if let Some(output_path) = &args.output {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_path)
            .with_context(|| format!("failed to open output log file: {}", output_path.display()))?;
        let file_err = file.try_clone().context("failed to clone file descriptor")?;
        cmd.stdout(Stdio::from(file));
        cmd.stderr(Stdio::from(file_err));
    } else {
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
    }

    unsafe {
        let raw_write = write_fd.as_raw_fd();
        let restore_mask = old_sigmask;
        cmd.pre_exec(move || {
            // Restore original signal mask in child process so child handles signals normally
            libc::sigprocmask(libc::SIG_SETMASK, &restore_mask, std::ptr::null_mut());

            // NOTE (Platform Limitation):
            // `prctl(PR_SET_PDEATHSIG)` is a Linux-only kernel capability.
            // On Darwin (macOS) and BSD systems without a kernel-level pdeathsig equivalent,
            // process cleanup relies on the parent's synchronous sigwait supervisor thread.
            // If the parent process is killed by an uncatchable signal (SIGKILL) or crashes hard,
            // the spawned gateway child may be reparented to launchd/init.
            #[cfg(target_os = "linux")]
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                return Err(std::io::Error::last_os_error());
            }

            if libc::dup2(raw_write, 3) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            if raw_write != 3 {
                libc::close(raw_write);
            }
            Ok(())
        });
    }

    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) => {
            restore_signals(&old_sigmask);
            return Err(err).context("failed to spawn transit background process");
        }
    };
    drop(write_fd); // RAII drop closes write end in parent so EOF is delivered if child exits

    let child = Arc::new(Mutex::new(child));
    let stop_watcher = Arc::new(AtomicBool::new(false));

    // Spawn synchronous signal watcher thread using sigwait
    let watcher_handle = spawn_signal_watcher(child.clone(), stop_watcher.clone());

    let mut ready_file: std::fs::File = read_fd.into();
    if let Err(err) = wait_for_readiness(&child, &mut ready_file, Duration::from_secs(15)) {
        stop_watcher.store(true, Ordering::SeqCst);
        terminate_child(&child);
        restore_signals(&old_sigmask);
        return Err(err);
    }

    // Transit background process is now ready. Execute the target command.
    let mut target_iter = args.command.into_iter();
    let program = target_iter.next().unwrap();
    let mut target_cmd = Command::new(program);
    target_cmd
        .args(target_iter)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    unsafe {
        let restore_mask = old_sigmask;
        target_cmd.pre_exec(move || {
            libc::sigprocmask(libc::SIG_SETMASK, &restore_mask, std::ptr::null_mut());
            Ok(())
        });
    }

    let exit_status = target_cmd.status();

    // Notify watcher thread to stop monitoring
    stop_watcher.store(true, Ordering::SeqCst);

    // Clean up gateway child process
    terminate_child(&child);

    // Restore original signal mask
    restore_signals(&old_sigmask);

    // Unblock watcher thread if it's waiting
    #[cfg(unix)]
    unsafe {
        use std::os::unix::thread::JoinHandleExt;
        let thread_id = watcher_handle.as_pthread_t();
        libc::pthread_kill(thread_id, libc::SIGTERM);
    }
    let _ = watcher_handle.join();

    match exit_status {
        Ok(status) => {
            if !status.success() {
                let code = status.code().unwrap_or(1);
                // INVARIANT: All child processes and file descriptors managed in this scope
                // must be fully terminated or safely closed before `std::process::exit` is called,
                // because `std::process::exit` bypasses Rust stack unwinding and struct Drop implementations.
                std::process::exit(code);
            }
            Ok(())
        }
        Err(err) => {
            bail!("failed to execute target command: {}", err);
        }
    }
}

#[cfg(unix)]
fn block_termination_signals() -> Result<libc::sigset_t> {
    unsafe {
        let mut old_set = std::mem::zeroed();
        let mut set = std::mem::zeroed();
        libc::sigemptyset(&mut set);
        libc::sigaddset(&mut set, libc::SIGINT);
        libc::sigaddset(&mut set, libc::SIGTERM);
        if libc::pthread_sigmask(libc::SIG_BLOCK, &set, &mut old_set) != 0 {
            return Err(std::io::Error::last_os_error()).context("failed to block termination signals");
        }
        Ok(old_set)
    }
}

#[cfg(unix)]
fn restore_signals(old_set: &libc::sigset_t) {
    unsafe {
        libc::pthread_sigmask(libc::SIG_SETMASK, old_set, std::ptr::null_mut());
    }
}

#[cfg(unix)]
fn spawn_signal_watcher(
    child: Arc<Mutex<Child>>,
    stop_flag: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut set = unsafe { std::mem::zeroed() };
        unsafe {
            libc::sigemptyset(&mut set);
            libc::sigaddset(&mut set, libc::SIGINT);
            libc::sigaddset(&mut set, libc::SIGTERM);
        }
        let mut sig = 0;
        loop {
            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
            let res = unsafe { libc::sigwait(&set, &mut sig) };
            if res == 0 && (sig == libc::SIGINT || sig == libc::SIGTERM) {
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }
                // Step 1: Graceful SIGTERM notification
                {
                    let mut guard = match child.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    if guard.try_wait().ok().flatten().is_some() {
                        std::process::exit(128 + sig);
                    }
                    let child_pid = guard.id() as i32;
                    unsafe {
                        libc::kill(child_pid, libc::SIGTERM);
                    }
                }

                // Step 2: Bounded grace period (1000ms) with polling via Child::try_wait
                let deadline = Instant::now() + Duration::from_millis(1000);
                let mut reaped = false;
                while Instant::now() < deadline {
                    let status = {
                        let mut guard = match child.lock() {
                            Ok(g) => g,
                            Err(p) => p.into_inner(),
                        };
                        guard.try_wait().ok().flatten()
                    };
                    if status.is_some() {
                        reaped = true;
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }

                // Step 3: Escalation to SIGKILL if child ignored or hung on SIGTERM
                if !reaped {
                    let mut guard = match child.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    if guard.try_wait().ok().flatten().is_none() {
                        let _ = guard.kill();
                        let _ = guard.wait();
                    }
                }

                // Exit with conventional signal exit code
                std::process::exit(128 + sig);
            }
        }
    })
}

#[cfg(unix)]
fn wait_for_readiness(
    child: &Mutex<Child>,
    ready_file: &mut std::fs::File,
    timeout: Duration,
) -> Result<()> {
    use std::io::Read;
    let start = Instant::now();
    loop {
        {
            let mut guard = match child.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            if let Some(status) = guard.try_wait()? {
                bail!("transit background process exited prematurely with status: {}", status);
            }
        }
        let mut buf = [0u8; 1];
        match ready_file.read(&mut buf) {
            Ok(n) if n > 0 => return Ok(()),
            Ok(_) => {
                let mut guard = match child.lock() {
                    Ok(g) => g,
                    Err(p) => p.into_inner(),
                };
                if let Some(status) = guard.try_wait()? {
                    bail!("transit background process exited prematurely with status: {}", status);
                }
                return Ok(());
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                if start.elapsed() > timeout {
                    terminate_child(child);
                    bail!(
                        "timed out waiting for transit background process to become ready ({}s)",
                        timeout.as_secs()
                    );
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(err) => {
                terminate_child(child);
                return Err(err).context("failed to read readiness signal from child");
            }
        }
    }
}

#[cfg(unix)]
fn create_nonblocking_pipe() -> Result<(std::os::fd::OwnedFd, std::os::fd::OwnedFd)> {
    use std::os::fd::{FromRawFd, OwnedFd};
    let mut fds = [-1, -1];
    unsafe {
        if libc::pipe(fds.as_mut_ptr()) == -1 {
            return Err(std::io::Error::last_os_error()).context("failed to create readiness pipe");
        }
        // Set read end to non-blocking
        let flags = libc::fcntl(fds[0], libc::F_GETFL);
        if flags != -1 {
            libc::fcntl(fds[0], libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
        Ok((OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])))
    }
}

fn terminate_child(child: &Mutex<Child>) {
    let mut guard = match child.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if guard.try_wait().ok().flatten().is_some() {
        return;
    }
    let _ = guard.kill();
    let _ = guard.wait();
}
