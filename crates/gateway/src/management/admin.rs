use hyper::{Body, Response, StatusCode, header::CONTENT_TYPE};

use super::{drain::DrainWatcher, hyper_helpers};

pub struct Server(hyper_helpers::Server);

impl Server {
    pub async fn new(
        address: crate::Address,
        config: String,
        drain: DrainWatcher,
    ) -> anyhow::Result<Self> {
        let server = hyper_helpers::Server::bind("admin", address, drain, move |request| {
            let config = config.clone();
            async move {
                let (status, body, content_type) = match (request.method(), request.uri().path()) {
                    (&hyper::Method::GET, "/") => {
                        (StatusCode::OK, "transit admin\n".to_owned(), "text/plain")
                    }
                    (&hyper::Method::GET, "/config_dump") => {
                        (StatusCode::OK, config, "application/yaml")
                    }
                    _ => (
                        StatusCode::NOT_FOUND,
                        "not found\n".to_owned(),
                        "text/plain",
                    ),
                };
                Ok(Response::builder()
                    .status(status)
                    .header(CONTENT_TYPE, content_type)
                    .body(Body::from(body))
                    .expect("valid admin response"))
            }
        })
        .await?;
        Ok(Self(server))
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        self.0.spawn()
    }
}
