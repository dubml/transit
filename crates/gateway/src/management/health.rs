use hyper::{Body, Request, Response, StatusCode};

use super::{drain::DrainWatcher, hyper_helpers, readiness::Ready};

pub struct Server(hyper_helpers::Server);

impl Server {
    pub async fn new(
        address: crate::Address,
        ready: Ready,
        drain: DrainWatcher,
    ) -> anyhow::Result<Self> {
        let server = hyper_helpers::Server::bind(
            "readiness",
            address,
            drain,
            move |request: Request<Body>| {
                let ready = ready.clone();
                async move {
                    let (status, body) = match (request.method(), request.uri().path()) {
                        (&hyper::Method::GET, "/healthz/ready") => {
                            let pending = ready.pending();
                            if pending.is_empty() {
                                (StatusCode::OK, "ready\n".to_owned())
                            } else {
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    format!("not ready, pending: {}\n", pending.join(", ")),
                                )
                            }
                        }
                        _ => (StatusCode::NOT_FOUND, "not found\n".to_owned()),
                    };
                    Ok(Response::builder()
                        .status(status)
                        .header(hyper::header::CONTENT_TYPE, "text/plain")
                        .body(Body::from(body))
                        .expect("valid readiness response"))
                }
            },
        )
        .await?;
        Ok(Self(server))
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        self.0.spawn()
    }
}
