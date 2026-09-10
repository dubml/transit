use super::XdsError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tonic::transport::{Certificate, ClientTlsConfig, Endpoint, Identity};

#[derive(Deserialize)]
struct Bootstrap {
    certificate_providers: HashMap<String, Provider>,
}

#[derive(Deserialize)]
struct Provider {
    plugin_name: String,
    config: Files,
}

#[derive(Deserialize)]
struct Files {
    certificate_file: PathBuf,
    private_key_file: PathBuf,
    ca_certificate_file: PathBuf,
}

pub(super) async fn configure(
    endpoint: Endpoint,
    bootstrap: Option<PathBuf>,
) -> Result<Endpoint, XdsError> {
    if endpoint.uri().scheme_str() != Some("https") {
        if bootstrap.is_some() {
            return Err(XdsError::Credentials(
                "workload credentials require an https ADS endpoint".into(),
            ));
        }
        return Ok(endpoint);
    }
    let path = bootstrap
        .ok_or_else(|| XdsError::Credentials("https ADS requires GRPC_XDS_BOOTSTRAP".into()))?;
    let bootstrap: Bootstrap = serde_json::from_slice(&read(&path).await?)
        .map_err(|_| XdsError::Credentials("invalid workload bootstrap JSON".into()))?;
    let provider = bootstrap
        .certificate_providers
        .get("default")
        .ok_or_else(|| XdsError::Credentials("missing default certificate provider".into()))?;
    if provider.plugin_name != "file_watcher" {
        return Err(XdsError::Credentials(
            "default certificate provider must use file_watcher".into(),
        ));
    }
    let files = &provider.config;
    let cert = read(&files.certificate_file).await?;
    let key = read(&files.private_key_file).await?;
    let ca = read(&files.ca_certificate_file).await?;
    endpoint
        .tls_config(
            ClientTlsConfig::new()
                .ca_certificate(Certificate::from_pem(ca))
                .identity(Identity::from_pem(cert, key)),
        )
        .map_err(|_| XdsError::Credentials("invalid TLS certificate, key, or trust bundle".into()))
}

pub(super) async fn configure_service_account(
    endpoint: Endpoint, root_ca: &Path,
) -> Result<Endpoint, XdsError> {
    if endpoint.uri().scheme_str() != Some("https") {
        return Err(XdsError::Credentials("ServiceAccount credentials require an https ADS endpoint".into()));
    }
    let ca = read(root_ca).await?;
    endpoint.tls_config(ClientTlsConfig::new().ca_certificate(Certificate::from_pem(ca)))
        .map_err(|_| XdsError::Credentials("invalid xDS trust bundle".into()))
}

async fn read(path: &Path) -> Result<Vec<u8>, XdsError> {
    tokio::fs::read(path)
        .await
        .map_err(|err| XdsError::Credentials(format!("cannot read {}: {err}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn credentials_cannot_downgrade_to_plaintext() {
        let endpoint = Endpoint::from_static("http://localhost:26010");
        assert!(configure(endpoint, Some("unused".into())).await.is_err());
    }

    #[tokio::test]
    async fn secure_ads_requires_explicit_credentials() {
        let endpoint = Endpoint::from_static("https://localhost:26012");
        assert!(configure(endpoint.clone(), None).await.is_err());
        assert!(
            configure(endpoint, Some("/does-not-exist/bootstrap.json".into()))
                .await
                .is_err()
        );
    }
}
