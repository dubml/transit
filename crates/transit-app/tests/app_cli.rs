use std::process::{Command, Stdio};

#[test]
fn test_version_json_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_transit"))
        .arg("-V")
        .output()
        .expect("failed to execute transit -V");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("version output should be valid JSON");
    assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("transit-app"));
    assert!(json.get("version").is_some());
}

#[test]
fn test_validate_stdin_config() {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_transit"))
        .arg("-f")
        .arg("-")
        .arg("--validate")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn transit");

    let valid_yaml = "version: v1\nroutes: []\n";
    child.stdin.as_mut().unwrap().write_all(valid_yaml.as_bytes()).unwrap();

    let output = child.wait_with_output().expect("failed to wait on child");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_stealth_rejects_stdin_config() {
    let output = Command::new(env!("CARGO_BIN_EXE_transit"))
        .arg("stealth")
        .arg("-f")
        .arg("-")
        .arg("--")
        .arg("echo")
        .arg("hello")
        .output()
        .expect("failed to execute transit stealth");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("stealth mode does not support reading configuration from standard input"));
}

#[test]
fn test_stealth_runs_command_and_cleans_up() {
    let output = Command::new(env!("CARGO_BIN_EXE_transit"))
        .arg("stealth")
        .arg("--")
        .arg("echo")
        .arg("STEALTH_SUCCESS")
        .output()
        .expect("failed to execute transit stealth");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("STEALTH_SUCCESS"));
}

#[test]
fn test_stealth_parent_sigterm_terminates_cleanly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_transit"))
        .arg("stealth")
        .arg("--")
        .arg("sleep")
        .arg("60")
        .spawn()
        .expect("failed to spawn transit stealth");

    std::thread::sleep(std::time::Duration::from_millis(300));

    let start = std::time::Instant::now();
    let pid = child.id() as i32;
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    let status = child.wait().expect("failed to wait on child");
    let elapsed = start.elapsed();
    assert!(!status.success());
    // Must terminate cleanly via SIGTERM without waiting out the full 1000ms grace period + escalation
    assert!(
        elapsed < std::time::Duration::from_millis(800),
        "expected graceful termination within 800ms, took {:?}",
        elapsed
    );
}

#[test]
fn test_stealth_escalates_to_sigkill_if_child_ignores_sigterm() {
    // Target command traps SIGTERM and ignores it
    let mut child = Command::new(env!("CARGO_BIN_EXE_transit"))
        .arg("stealth")
        .arg("--")
        .arg("sh")
        .arg("-c")
        .arg("trap '' TERM; sleep 60")
        .spawn()
        .expect("failed to spawn transit stealth");

    std::thread::sleep(std::time::Duration::from_millis(300));

    let pid = child.id() as i32;
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }

    let status = child.wait().expect("failed to wait on child");
    assert!(!status.success());
}
