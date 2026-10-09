use std::{convert::Infallible, future::Future, pin::Pin};

use hyper::{
    Body, Request, Response, Server as HyperServer,
    service::{make_service_fn, service_fn},
};
use tracing::{info, warn};

use super::drain::DrainWatcher;

type ServerFuture = Pin<Box<dyn Future<Output = Result<(), hyper::Error>> + Send>>;

pub struct Server {
    name: &'static str,
    servers: Vec<(std::net::SocketAddr, ServerFuture)>,
}

impl Server {
    pub async fn bind<F, Fut>(
        name: &'static str,
        addrs: crate::Address,
        drain: DrainWatcher,
        handler: F,
    ) -> anyhow::Result<Self>
    where
        F: Fn(Request<Body>) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<Body>, Infallible>> + Send + 'static,
    {
        let mut servers = Vec::new();
        for addr in addrs.socket_addrs() {
            let service = make_service_fn({
                let handler = handler.clone();
                move |_| {
                    let handler = handler.clone();
                    async move { Ok::<_, Infallible>(service_fn(move |request| handler(request))) }
                }
            });
            let server = HyperServer::try_bind(&addr)?.serve(service);
            let server = server.with_graceful_shutdown(drain.clone().signaled());
            info!(component = name, address = %addr, "Listener started");
            let server: ServerFuture = Box::pin(server);
            servers.push((addr, server));
        }
        if servers.is_empty() {
            info!(component = name, "Listener disabled");
        }
        Ok(Self { name, servers })
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut tasks = tokio::task::JoinSet::new();
            for (address, server) in self.servers {
                let name = self.name;
                tasks.spawn(async move { (name, address, server.await) });
            }
            while let Some(result) = tasks.join_next().await {
                match result {
                    Ok((name, address, Err(error))) => {
                        warn!(component = name, address = %address, %error, "Listener stopped unexpectedly")
                    }
                    Ok((name, address, Ok(()))) => {
                        info!(component = name, address = %address, "Listener drained")
                    }
                    Err(error) => warn!(%error, "Listener task failed"),
                }
            }
        })
    }
}
