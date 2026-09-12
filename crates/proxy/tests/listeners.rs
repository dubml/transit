use hyper::{Body, Client, Request, Response};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpStream, sync::oneshot, task::JoinHandle};
use transit_core::{
    Cluster, Endpoint, Listener, ListenerProtocol, Route, RuntimeConfig, TlsSecret, VirtualHost,
    WeightedCluster,
};
use transit_proxy::{ProxyServer, ProxyState};

struct Task(JoinHandle<std::io::Result<()>>);
impl Drop for Task {
    fn drop(&mut self) {
        self.0.abort();
    }
}

fn reserve() -> std::net::TcpListener {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap()
}

fn listener(name: &str, bind: SocketAddr) -> Listener {
    Listener {
        name: name.into(),
        bind,
        protocol: ListenerProtocol::Http,
        virtual_hosts: vec![],
        tls_secret: None,
        security: Default::default(),
    }
}

fn start(state: &ProxyState) -> (Task, oneshot::Sender<()>) {
    state.require_configured_listeners();
    let (tx, rx) = oneshot::channel();
    let proxy = ProxyServer::new(state.clone());
    (
        Task(tokio::spawn(proxy.serve_configured_with_shutdown(
            Duration::from_secs(1),
            async {
                let _ = rx.await;
            },
        ))),
        tx,
    )
}

async fn until(mut predicate: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !predicate() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("condition did not converge");
}

async fn closed(address: SocketAddr) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while TcpStream::connect(address).await.is_ok() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("listener remained open");
}

fn upstream(body: &'static str) -> (SocketAddr, Task) {
    let socket = reserve();
    let address = socket.local_addr().unwrap();
    socket.set_nonblocking(true).unwrap();
    let service = hyper::service::make_service_fn(move |_| async move {
        Ok::<_, std::convert::Infallible>(hyper::service::service_fn(move |_| async move {
            Ok::<_, std::convert::Infallible>(Response::new(Body::from(body)))
        }))
    });
    (
        address,
        Task(tokio::spawn(async move {
            hyper::Server::from_tcp(socket)
                .unwrap()
                .serve(service)
                .await
                .map_err(std::io::Error::other)
        })),
    )
}

fn add_route(config: &mut RuntimeConfig, address: SocketAddr, upstream: SocketAddr, name: &str) {
    let mut port = listener(name, address);
    port.virtual_hosts.push(VirtualHost {
        name: name.into(),
        domains: vec!["*".into()],
        routes: vec![Route {
            name: name.into(),
            matches: vec![],
            weighted_clusters: vec![WeightedCluster {
                name: name.into(),
                weight: 1,
            }],
        }],
    });
    config.listeners.push(port);
    config.clusters.push(Cluster {
        name: name.into(),
        endpoints: vec![Endpoint {
            address: upstream.ip().to_string(),
            port: upstream.port(),
            healthy: true,
            node_name: None,
        }],
        http2: false,
        tls: None,
        circuit_breaker: None,
        outlier_detection: None,
    });
}

async fn get(address: SocketAddr) -> String {
    let client = Client::builder()
        .pool_max_idle_per_host(0)
        .build_http::<Body>();
    let response = client
        .get(format!("http://{address}/").parse().unwrap())
        .await
        .unwrap();
    assert!(response.status().is_success());
    String::from_utf8(
        hyper::body::to_bytes(response.into_body())
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}

#[tokio::test]
async fn configured_http_ports_route_independently_and_follow_updates() {
    let state = ProxyState::new();
    let occupied = reserve();
    let first = occupied.local_addr().unwrap();
    let second = reserve().local_addr().unwrap();
    let (one, _one_task) = upstream("one");
    let (two, _two_task) = upstream("two");
    let mut config = RuntimeConfig::empty("initial");
    add_route(&mut config, first, one, "first");
    add_route(&mut config, second, two, "second");
    state.apply_config(config.clone()).unwrap();
    let (mut task, stop) = start(&state);
    assert!(!state.readiness().ready);
    until(|| {
        state
            .readiness()
            .conflicts
            .iter()
            .any(|c| c.kind == "listener-bind")
    })
    .await;
    drop(occupied);
    until(|| state.readiness().ready).await;
    assert_eq!(get(first).await, "one");
    assert_eq!(get(second).await, "two");

    config.version = "remove-first".into();
    config.listeners.remove(0);
    state.apply_config(config.clone()).unwrap();
    assert!(
        !state.readiness().ready,
        "a newly published revision must wait for sockets"
    );
    until(|| state.readiness().ready).await;
    closed(first).await;
    assert_eq!(get(second).await, "two");

    config.version = "add-first".into();
    let mut restored = listener("restored", first);
    restored.virtual_hosts = config.listeners[0].virtual_hosts.clone();
    config.listeners.push(restored);
    state.apply_config(config).unwrap();
    until(|| state.readiness().ready).await;
    assert_eq!(get(first).await, "two");
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(3), &mut task.0)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(!state.readiness().ready);
    closed(first).await;
    closed(second).await;
}

fn certificate() -> (TlsSecret, Vec<u8>) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let pem = cert.serialize_pem().unwrap();
    let der = rustls_pemfile::certs(&mut pem.as_bytes())
        .unwrap()
        .remove(0);
    (
        TlsSecret {
            name: "serving".into(),
            certificate_chain_pem: pem,
            private_key_pem: cert.serialize_private_key_pem(),
            trusted_ca_pem: None,
        },
        der,
    )
}

async fn connect_tls(
    address: SocketAddr,
    roots: &[Vec<u8>],
) -> tokio_rustls::client::TlsStream<TcpStream> {
    let mut trust = rustls::RootCertStore::empty();
    for cert in roots {
        trust.add(&rustls::Certificate(cert.clone())).unwrap();
    }
    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(trust)
        .with_no_client_auth();
    tokio_rustls::TlsConnector::from(Arc::new(config))
        .connect(
            "localhost".try_into().unwrap(),
            TcpStream::connect(address).await.unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn configured_tls_waits_for_secrets_rotates_and_withdraws_invalid_material() {
    let state = ProxyState::new();
    let address = reserve().local_addr().unwrap();
    let mut config = RuntimeConfig::empty("missing-secret");
    let mut port = listener("tls", address);
    port.protocol = ListenerProtocol::Https;
    port.tls_secret = Some("serving".into());
    config.listeners.push(port);
    state.apply_config(config.clone()).unwrap();
    let (mut task, stop) = start(&state);
    until(|| {
        state
            .readiness()
            .conflicts
            .iter()
            .any(|c| c.kind == "listener-config")
    })
    .await;
    closed(address).await;
    let (first, first_der) = certificate();
    let (second, second_der) = certificate();
    let roots = vec![first_der.clone(), second_der.clone()];
    config.secrets.push(first);
    state.apply_config(config.clone()).unwrap();
    until(|| state.readiness().ready).await;
    let original = connect_tls(address, &roots).await;
    assert_eq!(
        original.get_ref().1.peer_certificates().unwrap()[0].0,
        first_der
    );

    config.secrets[0] = second;
    state.apply_config(config.clone()).unwrap();
    until(|| state.readiness().ready).await;
    let rotated = connect_tls(address, &roots).await;
    assert_eq!(
        rotated.get_ref().1.peer_certificates().unwrap()[0].0,
        second_der
    );
    drop(rotated);
    let (mut sender, connection) = hyper::client::conn::handshake(original).await.unwrap();
    let connection_task = tokio::spawn(connection);
    let response = sender
        .send_request(
            Request::builder()
                .uri("/")
                .header("host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        404,
        "the existing TLS connection survives rotation"
    );
    drop(sender);
    connection_task.abort();

    config.secrets.clear();
    state.apply_config(config.clone()).unwrap();
    until(|| {
        state
            .readiness()
            .conflicts
            .iter()
            .any(|c| c.kind == "listener-config")
    })
    .await;
    closed(address).await;
    let (mut invalid, _) = certificate();
    invalid.private_key_pem = "invalid key".into();
    config.secrets.push(invalid);
    state.apply_config(config.clone()).unwrap();
    until(|| {
        state
            .readiness()
            .conflicts
            .iter()
            .any(|c| c.kind == "listener-tls")
    })
    .await;
    closed(address).await;
    config.secrets[0] = certificate().0;
    state.apply_config(config).unwrap();
    until(|| state.readiness().ready).await;
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(3), &mut task.0)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    closed(address).await;
}
