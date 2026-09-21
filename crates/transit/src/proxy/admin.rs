use axum::body::Body;
use axum::http::{header, HeaderMap, Method, Response, StatusCode};
use serde_json::json;

use crate::proxy::ProxyServer;

pub async fn handle_admin_request(
    server: &ProxyServer,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
    body_bytes: &[u8],
) -> Option<Response<Body>> {
    let clean_path = path.trim_start_matches('/');

    // 1. Health check
    if clean_path == "healthz" {
        return Some(json_response(StatusCode::OK, &json!({"status": "ok"})));
    }

    // Debug / runtime inspection endpoints
    if clean_path == "debug/config" {
        let cfg = server.current_config();
        return Some(json_response(StatusCode::OK, &*cfg));
    }

    if clean_path == "debug/llm" {
        let configured = server.access_settings().configured();
        let mode = if configured { "token" } else { "local" };
        let cfg = server.current_config();
        let llm_cfg = cfg.llm.as_ref();
        let providers = llm_cfg.map(|l| l.providers());
        let subscriptions = llm_cfg.map(|l| l.subscriptions());
        let accounts = server.llm_accounts().list();
        return Some(json_response(
            StatusCode::OK,
            &json!({
                "management_enabled": true,
                "management_mode": mode,
                "accounts": accounts,
                "config_version": cfg.version.clone().unwrap_or_default(),
                "providers": providers,
                "subscriptions": subscriptions,
            }),
        ));
    }

    if clean_path == "debug/services" {
        let cfg = server.current_config();
        return Some(json_response(
            StatusCode::OK,
            &json!({
                "listeners": cfg.listeners(),
                "routes": cfg.routes,
            }),
        ));
    }

    // 2. Access status
    if clean_path == "admin/access/status" {
        let configured = server.access_settings().configured();
        let mode = if configured { "token" } else { "local" };
        return Some(json_response(
            StatusCode::OK,
            &json!({
                "configured": configured,
                "enabled": true,
                "management_mode": mode,
            }),
        ));
    }

    // 3. Admin session token
    if clean_path == "admin/session" {
        let token = server
            .access_settings()
            .local_token()
            .unwrap_or_else(|| "transit-local-session".to_string());
        return Some(json_response(StatusCode::OK, &json!({ "token": token })));
    }

    // 4. Access settings view / save
    if clean_path == "admin/access" {
        if method == Method::GET {
            if let Some(view) = server.access_settings().view() {
                return Some(json_response(StatusCode::OK, &view));
            }
            return Some(json_response(
                StatusCode::OK,
                &json!({
                    "writable": true,
                    "revision": 1,
                    "config": {
                        "host": "localhost",
                        "port": crate::HTTP_LISTENER_PORT,
                        "auth_dir": ".transit/auth",
                        "tls": false
                    },
                    "active": {
                        "host": "localhost",
                        "port": crate::HTTP_LISTENER_PORT,
                        "auth_dir": ".transit/auth",
                        "tls": false
                    },
                    "restart_required": false,
                    "secret_configured": false,
                    "config_file": "config.yaml"
                }),
            ));
        }
    }

    // 5. LLM local session token
    if clean_path == "admin/llm/session" {
        let token = server
            .llm_accounts()
            .local_session_token()
            .unwrap_or_else(|| "transit-llm-local-session".to_string());
        return Some(json_response(StatusCode::OK, &json!({ "token": token })));
    }

    // 6. LLM accounts list
    if clean_path == "admin/llm/accounts" || clean_path == "admin/llm/accounts/" {
        if method == Method::GET {
            let list = server.llm_accounts().list();
            return Some(json_response(StatusCode::OK, &list));
        }
    }

    // 7. LLM OAuth start
    if clean_path == "admin/llm/oauth/start" && method == Method::POST {
        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(body_bytes) {
            let provider = val
                .get("provider")
                .and_then(|p| p.as_str())
                .unwrap_or("codex");
            let backend = val.get("backend").and_then(|b| b.as_str()).unwrap_or("");
            match server.llm_accounts().start_login(provider, backend) {
                Ok(login) => return Some(json_response(StatusCode::OK, &login)),
                Err(err) => {
                    return Some(json_response(
                        StatusCode::BAD_REQUEST,
                        &json!({ "error": err }),
                    ))
                }
            }
        }
    }

    // 8. LLM OAuth status / cancel
    if clean_path.starts_with("admin/llm/oauth/") {
        let oauth_id = clean_path.trim_start_matches("admin/llm/oauth/");
        if method == Method::GET {
            let status = server.llm_accounts().login_status(oauth_id);
            return Some(json_response(StatusCode::OK, &status));
        }
        if method == Method::DELETE {
            server.llm_accounts().cancel_login(oauth_id).await;
            return Some(json_response(
                StatusCode::OK,
                &json!({ "status": "cancelled" }),
            ));
        }
    }

    // 9. LLM Account detail / models / quota / delete
    if clean_path.starts_with("admin/llm/accounts/") {
        let sub = clean_path.trim_start_matches("admin/llm/accounts/");
        if let Some((account_id, action)) = sub.split_once('/') {
            if action == "models" && method == Method::GET {
                match server.llm_accounts().account_models(account_id).await {
                    Ok(models) => return Some(json_response(StatusCode::OK, &models)),
                    Err(err) => {
                        return Some(json_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            &json!({ "error": err }),
                        ))
                    }
                }
            }
            if action == "quota" && method == Method::GET {
                match server.llm_accounts().account_quota(account_id).await {
                    Ok(quota) => return Some(json_response(StatusCode::OK, &quota)),
                    Err(err) => {
                        return Some(json_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            &json!({ "error": err }),
                        ))
                    }
                }
            }
        } else {
            let account_id = sub;
            if method == Method::GET {
                if let Some(account) = server.llm_accounts().get(account_id) {
                    return Some(json_response(StatusCode::OK, &account.summary()));
                }
                return Some(json_response(
                    StatusCode::NOT_FOUND,
                    &json!({ "error": "Account not found" }),
                ));
            }
            if method == Method::DELETE {
                match server.llm_accounts().delete(account_id, 0).await {
                    Ok(()) => {
                        return Some(json_response(
                            StatusCode::OK,
                            &json!({ "status": "deleted" }),
                        ))
                    }
                    Err(err) => {
                        return Some(json_response(
                            StatusCode::BAD_REQUEST,
                            &json!({ "error": err }),
                        ))
                    }
                }
            }
        }
    }

    let _ = headers;
    None
}

fn json_response<T: serde::Serialize>(status: StatusCode, data: &T) -> Response<Body> {
    let body = serde_json::to_vec(data).unwrap_or_default();
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .body(Body::from(body))
        .unwrap()
}