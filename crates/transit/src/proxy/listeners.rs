use super::{proxy_request, ProxyServer};
use axum::{routing::any, Router};
use futures_util::StreamExt;
use std::{collections::BTreeMap, io, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{sync::watch, task::JoinSet};
use crate::{ConfigConflict, ListenerProtocol, RuntimeConfig};

#[derive(Clone, PartialEq, Eq)]
enum Transport {
    Http,
    Https,
}

struct RunningListener {
    generation: u64,
    transport: Transport,
    tls: Option<watch::Sender<Arc<rustls::ServerConfig>>>,
    stop: watch::Sender<bool>,
}

impl ProxyServer {
    /// Follows configuration and binds sockets. Unchanged sockets are reused.
    pub async fn serve_configured_with_shutdown(
        self,
        _drain_timeout: Duration,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> io::Result<()> {
        let mut running = BTreeMap::<SocketAddr, RunningListener>::new();
        let mut tasks = JoinSet::new();
        let mut generation = 0_u64;
        let mut healthy_version = None;
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
                                healthy_version = None;
                            }
                            if let Err(error) = result {
                                tracing::warn!(%address, %error, "configured listener stopped");
                            }
                        }
                        Some(Err(error)) => {
                            return Err(io::Error::other(error));
                        }
                        None => {}
                    }
                }
                _ = interval.tick() => {
                    let config = &self.config;
                    if healthy_version == config.version { continue; }
                    let (desired, mut errors) = desired_listeners(config);
                    running.retain(|address, listener| {
                        let keep = desired.get(address).is_some_and(|transport| {
                            matches!((transport, &listener.transport), (Transport::Http, Transport::Http) | (Transport::Https, Transport::Https))
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
                            Transport::Https => None,
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
                        let (stop_tx, stop_rx) = watch::channel(false);
                        let (tls_tx, tls_rx) = match tls {
                            Some(config) => {
                                let (tx, rx) = watch::channel(config);
                                (Some(tx), Some(rx))
                            }
                            None => (None, None),
                        };
                        generation += 1;
                        let id = generation;
                        let server_clone = self.clone();
                        let app = Router::new()
                            .fallback(any(proxy_request))
                            .with_state(server_clone);
                        tasks.spawn(async move {
                            let result = serve_socket(app, socket, tls_rx, stop_rx).await;
                            (address, id, result)
                        });
                        running.insert(address, RunningListener {
                            generation: id,
                            transport,
                            tls: tls_tx,
                            stop: stop_tx,
                        });
                    }
                    healthy_version = errors.is_empty().then_some(config.version.clone()).flatten();
                }
            }
        }
        for listener in running.values() {
            listener.stop.send_replace(true);
        }
        while tasks.join_next().await.is_some() {}
        Ok(())
    }
}

fn desired_listeners(
    config: &RuntimeConfig,
) -> (BTreeMap<SocketAddr, Transport>, Vec<ConfigConflict>) {
    let mut desired = BTreeMap::new();
    let mut invalid = std::collections::BTreeSet::new();
    let mut errors = Vec::new();
    for port_cfg in &config.ports {
        let addr = SocketAddr::from(([0, 0, 0, 0], port_cfg.default_port));
        if port_cfg.default_port == 0 {
            invalid.insert(addr);
            errors.push(ConfigConflict::new(
                "listener-config",
                "Port requires a nonzero port number",
            ));
            continue;
        }
        for listener in &port_cfg.listeners {
            let transport = match listener.protocol {
                ListenerProtocol::Http | ListenerProtocol::Tcp | ListenerProtocol::Grpc => Transport::Http,
                ListenerProtocol::Https | ListenerProtocol::Tls => Transport::Https,
            };
            if desired
                .insert(addr, transport.clone())
                .is_some_and(|old| old != transport)
            {
                invalid.insert(addr);
                errors.push(ConfigConflict::new(
                    "listener-bind-conflict",
                    format!("Listeners at {} require incompatible transports", addr),
                ));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ListenerConfig, ListenerProtocol, PortConfig, RuntimeConfig};

    #[test]
    fn test_desired_listeners_empty_produces_no_conflicts() {
        let config = RuntimeConfig::empty("v1");
        let (desired, errors) = desired_listeners(&config);
        assert!(desired.is_empty());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_desired_listeners_parses_port_listeners() {
        let config = RuntimeConfig {
            version: Some("v1".into()),
            ports: vec![PortConfig {
                default_port: 6010,
                listeners: vec![ListenerConfig {
                    name: "http-listener".into(),
                    protocol: ListenerProtocol::Http,
                    routes: vec![],
                }],
            }],
        };
        let (desired, errors) = desired_listeners(&config);
        assert_eq!(desired.len(), 1);
        assert!(errors.is_empty());
        let addr = SocketAddr::from(([0, 0, 0, 0], 6010));
        assert!(desired.contains_key(&addr));
    }
}
