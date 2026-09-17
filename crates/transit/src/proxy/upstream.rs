//! Upstream HTTP clients: client pooling for plain, web (HTTPS), and HTTP/2.

use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use hyper::client::HttpConnector;
use hyper::Client;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};

type PlainClient = Client<HttpConnector, Body>;
type WebClient = Client<HttpsConnector<HttpConnector>, Body>;

#[derive(Clone)]
pub(super) struct UpstreamClients {
    plaintext: PlainClient,
    web: WebClient,
    h2: WebClient,
}

impl UpstreamClients {
    pub(super) fn from_env() -> Self {
        let web_connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();
        let h2_connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http2()
            .build();
        Self {
            plaintext: Client::new(),
            web: Client::builder().build::<_, Body>(web_connector),
            h2: Client::builder()
                .http2_only(true)
                .build::<_, Body>(h2_connector),
        }
    }

    pub(super) async fn request_h2(
        &self,
        req: Request<Body>,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        self.h2
            .request(req)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))
    }

    pub(super) async fn request_plain(
        &self,
        req: Request<Body>,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        self.plaintext
            .request(req)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))
    }

    #[allow(dead_code)]
    pub(super) async fn request_web(
        &self,
        req: Request<Body>,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        self.web
            .request(req)
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))
    }
}
