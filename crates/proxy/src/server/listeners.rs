use super::{proxy_request, ProxyServer};
use axum::{routing::any, Router};
use futures_util::StreamExt;
use std::{collections::BTreeMap, io, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{sync::watch, task::JoinSet};
use transit_core::{ConfigConflict, ConfigSnapshot, ListenerProtocol, TlsSecret};

#[derive(Clone, PartialEq, Eq)]
enum Transport {
    Http,
    Https(Arc<TlsSecret>),
}

struct RunningListener {
    generation: u64,
    transport: Transport,
    tls: Option<watch::Sender<Arc<rustls::ServerConfig>>>,
    stop: watch::Sender<bool>,
}

impl ProxyServer {
    /// Follows published configuration without changing Store ownership or its
    /// update path. Polling coalesces bursts and keeps runtime dependencies out
    /// of the configuration core; unchanged sockets and TLS keys are reused.
    pub async fn serve_configured_with_shutdown(
        self,
        drain_timeout: Duration,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> io::Result<()> {
        self.state.require_configured_listeners();
        let mut running = BTreeMap::<SocketAddr, RunningListener>::new();
        let mut tasks = JoinSet::new();
        let mut generation = 0_u64;
        let mut healthy_revision = None;
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        tokio::pin!(shutdown);
        loop {
            tokio::select! {
                _ = &mut shutdown => break,
                result = tasks.join_next(), if !tasks.is_empty() => {
                    match result {
                        Some(Ok((address, id, result))) => {
                            if running.get(&address).is_some_and(|listener| listener.generation == id) {
                                running.remove(&address);
                                healthy_revision = None;
                                self.state.set_listener_readiness(None, vec![ConfigConflict::new(
                                    "listener-stopped", format!("Listener at {address} stopped unexpectedly"),
                                )]);
                            }
                            if let Err(error) = result {
                                tracing::warn!(%address, %error, "configured listener stopped");
                            }
                        }
                        Some(Err(error)) => {
                            self.state.set_listener_readiness(None, Vec::new());
                            return Err(io::Error::other(error));
                        }
                        None => {}
                    }
                }
                _ = interval.tick() => {
                    let snapshot = self.state.store().snapshot();
                    if healthy_revision == Some(snapshot.revision()) { continue; }
                    let (desired, mut errors) = desired_listeners(&snapshot);
                    running.retain(|address, listener| {
                        let keep = desired.get(address).is_some_and(|transport| {
                            matches!((transport, &listener.transport), (Transport::Http, Transport::Http) | (Transport::Https(_), Transport::Https(_)))
                        });
                        if !keep { listener.stop.send_replace(true); }
                        keep
                    });
                    for (address, transport) in desired {
                        if running.get(&address).is_some_and(|listener| listener.transport == transport) {
                            continue;
                        }
                        let tls = match &transport {
                            Transport::Http => None,
                            Transport::Https(secret) => match crate::access_settings::tls_config_from_pem(
                                secret.certificate_chain_pem.as_bytes(), secret.private_key_pem.as_bytes(),
                            ) {
                                Ok(config) => Some(config),
                                Err(_) => {
                                    if let Some(listener) = running.remove(&address) { listener.stop.send_replace(true); }
                                    errors.push(ConfigConflict::new("listener-tls", format!("Listener at {address} has an invalid TLS certificate or private key")));
                                    continue;
                                }
                            },
                        };
                        if let Some(listener) = running.get_mut(&address) {
                            if let (Some(sender), Some(config)) = (&listener.tls, tls) {
                                sender.send_replace(config);
                                listener.transport = transport;
                            }
                            continue;
                        }
                        let socket = match std::net::TcpListener::bind(address).and_then(|socket| {
                            socket.set_nonblocking(true)?;
                            Ok(socket)
                        }) {
                            Ok(socket) => socket,
                            Err(error) => {
                                errors.push(ConfigConflict::new("listener-bind", format!("Cannot bind listener at {address}: {error}")));
                                continue;
                            }
                        };
                        generation += 1;
                        let id = generation;
                        let (stop, stop_rx) = watch::channel(false);
                        let (tls_tx, tls_rx) = match tls {
                            Some(config) => {
                                let (tx, rx) = watch::channel(config);
                                (Some(tx), Some(rx))
                            }
                            None => (None, None),
                        };
                        let mut server = self.clone();
                        server.listener_port = address.port();
                        let app = Router::new().fallback(any(proxy_request)).with_state(server);
                        tasks.spawn(async move {
                            let serving = serve_socket(app, socket, tls_rx, stop_rx.clone());
                            tokio::pin!(serving);
                            let result = tokio::select! {
                                result = &mut serving => result,
                                _ = stopped(stop_rx) => tokio::time::timeout(drain_timeout, &mut serving)
                                    .await.unwrap_or(Ok(())),
                            };
                            (address, id, result)
                        });
                        running.insert(address, RunningListener { generation: id, transport, tls: tls_tx, stop });
                    }
                    if running.is_empty() && errors.is_empty() {
                        errors.push(ConfigConflict::new("listener-pending", "No configured listener is bound"));
                    }
                    healthy_revision = errors.is_empty().then_some(snapshot.revision());
                    self.state.set_listener_readiness(Some(snapshot.revision()), errors);
                }
            }
        }
        self.state.set_listener_readiness(None, Vec::new());
        for listener in running.values() {
            listener.stop.send_replace(true);
        }
        while tasks.join_next().await.is_some() {}
        Ok(())
    }
}

fn desired_listeners(
    snapshot: &ConfigSnapshot,
) -> (BTreeMap<SocketAddr, Transport>, Vec<ConfigConflict>) {
    let mut desired = BTreeMap::new();
    let mut invalid = std::collections::BTreeSet::new();
    let mut errors = Vec::new();
    for listener in snapshot.listeners() {
        let transport = match (listener.protocol, &listener.tls_secret) {
            (ListenerProtocol::Http, None) => Some(Transport::Http),
            (ListenerProtocol::Https, Some(name)) => {
                snapshot.secret(name).cloned().map(Transport::Https)
            }
            _ => None,
        };
        if listener.bind.port() == 0 || transport.is_none() {
            invalid.insert(listener.bind);
            errors.push(ConfigConflict::new(
                "listener-config",
                format!(
                    "Listener {} requires a nonzero port and TLS material matching its protocol",
                    listener.name
                ),
            ));
            continue;
        }
        let transport = transport.unwrap();
        if desired
            .insert(listener.bind, transport.clone())
            .is_some_and(|old| old != transport)
        {
            invalid.insert(listener.bind);
            errors.push(ConfigConflict::new(
                "listener-bind-conflict",
                format!(
                    "Listeners at {} require incompatible transports or certificates",
                    listener.bind
                ),
            ));
        }
    }
    for address in invalid {
        desired.remove(&address);
    }
    (desired, errors)
}

async fn stopped(mut rx: watch::Receiver<bool>) {
    while !*rx.borrow_and_update() {
        if rx.changed().await.is_err() {
            return;
        }
    }
}

async fn serve_socket(
    app: Router,
    socket: std::net::TcpListener,
    tls: Option<watch::Receiver<Arc<rustls::ServerConfig>>>,
    shutdown: watch::Receiver<bool>,
) -> io::Result<()> {
    let Some(tls) = tls else {
        return axum::Server::from_tcp(socket)
            .map_err(io::Error::other)?
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(stopped(shutdown))
            .await
            .map_err(io::Error::other);
    };
    let listener = tokio::net::TcpListener::from_std(socket)?;
    let incoming = tokio_stream::wrappers::TcpListenerStream::new(listener)
        .map(move |stream| {
            // Read the latest certificate for each new connection; established
            // TLS connections continue draining with their original session.
            let acceptor = tokio_rustls::TlsAcceptor::from(tls.borrow().clone());
            async move {
                tokio::time::timeout(Duration::from_secs(10), acceptor.accept(stream?))
                    .await
                    .map_err(io::Error::other)?
            }
        })
        .buffer_unordered(64)
        .filter_map(|result| async { result.ok().map(Ok::<_, io::Error>) });
    let make_service = hyper::service::make_service_fn(
        move |connection: &tokio_rustls::server::TlsStream<tokio::net::TcpStream>| {
            let app = app.clone();
            let peer = connection.get_ref().0.peer_addr();
            async move { peer.map(|peer| app.layer(axum::Extension(axum::extract::ConnectInfo(peer)))) }
        },
    );
    axum::Server::builder(hyper::server::accept::from_stream(Box::pin(incoming)))
        .serve(make_service)
        .with_graceful_shutdown(stopped(shutdown))
        .await
        .map_err(io::Error::other)
}
