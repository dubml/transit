use axum::body::Body;
use axum::http::{header, Response, StatusCode};

pub fn handle_ui_request(path: &str) -> Option<Response<Body>> {
    let clean_path = path.trim_start_matches('/');
    if clean_path == "ui" {
        return Some(
            Response::builder()
                .status(StatusCode::PERMANENT_REDIRECT)
                .header(header::LOCATION, "/ui/")
                .body(Body::empty())
                .unwrap(),
        );
    }
    if clean_path == "ui/" || clean_path == "ui/index.html" {
        return Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(include_str!("../../../ui/index.html")))
                .unwrap(),
        );
    }
    match clean_path {
        "ui/src/css/main.css" | "src/css/main.css" => Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
                .body(Body::from(include_str!("../../../ui/src/css/main.css")))
                .unwrap(),
        ),
        "ui/src/css/llm.css" | "src/css/llm.css" => Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
                .body(Body::from(include_str!("../../../ui/src/css/llm.css")))
                .unwrap(),
        ),
        "ui/src/css/configuration.css" | "src/css/configuration.css" => Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
                .body(Body::from(include_str!(
                    "../../../ui/src/css/configuration.css"
                )))
                .unwrap(),
        ),
        "ui/src/js/app.js" | "src/js/app.js" => Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
                .body(Body::from(include_str!("../../../ui/src/js/app.js")))
                .unwrap(),
        ),
        "ui/src/js/llm.js" | "src/js/llm.js" => Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
                .body(Body::from(include_str!("../../../ui/src/js/llm.js")))
                .unwrap(),
        ),
        "ui/src/js/configuration.js" | "src/js/configuration.js" => Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
                .body(Body::from(include_str!(
                    "../../../ui/src/js/configuration.js"
                )))
                .unwrap(),
        ),
        "assets/transit-logo.svg" | "ui/assets/transit-logo.svg" | "logo/logo.svg" => Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/svg+xml")
                .body(Body::from(include_str!("../../../logo/logo.svg")))
                .unwrap(),
        ),
        "assets/transit-mark.svg" | "ui/assets/transit-mark.svg" | "logo/mark.svg" => Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/svg+xml")
                .body(Body::from(include_str!("../../../logo/mark.svg")))
                .unwrap(),
        ),
        _ if clean_path.starts_with("ui/") => Some(
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(include_str!("../../../ui/index.html")))
                .unwrap(),
        ),
        _ => None,
    }
}
