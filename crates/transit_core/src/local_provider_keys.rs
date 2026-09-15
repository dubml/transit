use crate::SecretKeyReference;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs, io,
    path::{Path, PathBuf},
};

const LOCAL_PROVIDER_NAMESPACE: &str = "transit.local";
const LOCAL_PROVIDER_KEY: &str = "api-key";
const MAX_KEY_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalProviderKeyFile {
    #[serde(default = "key_file_version")]
    version: u8,
    #[serde(default)]
    providers: BTreeMap<String, String>,
}

impl Default for LocalProviderKeyFile {
    fn default() -> Self {
        Self {
            version: key_file_version(),
            providers: BTreeMap::new(),
        }
    }
}

fn key_file_version() -> u8 {
    1
}

pub fn local_provider_key_reference(provider: &str) -> SecretKeyReference {
    SecretKeyReference {
        namespace: LOCAL_PROVIDER_NAMESPACE.into(),
        name: provider.into(),
        key: LOCAL_PROVIDER_KEY.into(),
    }
}

pub fn local_provider_key_path(runtime_config: &Path) -> io::Result<PathBuf> {
    let parent = runtime_config
        .parent()
        .ok_or_else(|| io::Error::other("Runtime configuration has no parent directory"))?;
    let name = runtime_config
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::other("Runtime configuration file name is invalid"))?;
    Ok(parent.join(format!(".{name}.provider-keys.json")))
}

pub fn load_local_provider_keys(
    runtime_config: &Path,
) -> io::Result<HashMap<SecretKeyReference, String>> {
    let path = local_provider_key_path(runtime_config)?;
    let raw = match fs::read(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(HashMap::new()),
        Err(error) => return Err(error),
    };
    if raw.len() as u64 > MAX_KEY_FILE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Local provider key file exceeds 1 MiB",
        ));
    }
    let file: LocalProviderKeyFile = serde_json::from_slice(&raw).map_err(io::Error::other)?;
    if file.version != key_file_version() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unsupported local provider key file version",
        ));
    }
    let mut keys = HashMap::new();
    for (provider, key) in file.providers {
        validate_provider_name(&provider)?;
        validate_key(&key)?;
        keys.insert(local_provider_key_reference(&provider), key);
    }
    Ok(keys)
}

pub fn save_local_provider_key(runtime_config: &Path, provider: &str, key: &str) -> io::Result<()> {
    validate_provider_name(provider)?;
    validate_key(key)?;
    let path = local_provider_key_path(runtime_config)?;
    let mut file = match fs::read(&path) {
        Ok(raw) => serde_json::from_slice(&raw).map_err(io::Error::other)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => LocalProviderKeyFile::default(),
        Err(error) => return Err(error),
    };
    if file.version != key_file_version() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unsupported local provider key file version",
        ));
    }
    file.providers.insert(provider.into(), key.into());
    write_key_file(&path, &file)
}

fn write_key_file(path: &Path, file: &LocalProviderKeyFile) -> io::Result<()> {
    use std::io::Write;

    let raw = serde_json::to_vec_pretty(file).map_err(io::Error::other)?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Local provider key file has no parent directory"))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| io::Error::other("Local provider key file name is invalid"))?;
    let temporary = parent.join(format!(".{name}.tmp"));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> io::Result<()> {
        let mut handle = options.open(&temporary)?;
        handle.write_all(&raw)?;
        handle.write_all(b"\n")?;
        handle.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn validate_provider_name(provider: &str) -> io::Result<()> {
    if provider.is_empty()
        || provider.len() > 80
        || !provider
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid local provider key name",
        ));
    }
    Ok(())
}

fn validate_key(key: &str) -> io::Result<()> {
    if key.is_empty() || key.len() > 16 * 1024 || key.chars().any(char::is_control) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid local provider API key",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_provider_keys_round_trip_without_touching_runtime_config() {
        let directory = std::env::temp_dir().join(format!(
            "transit-local-provider-keys-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let config = directory.join("runtime.json");
        save_local_provider_key(&config, "openai-prod", "sk-test").unwrap();
        assert_eq!(
            load_local_provider_keys(&config)
                .unwrap()
                .get(&local_provider_key_reference("openai-prod")),
            Some(&"sk-test".to_string())
        );
        let key_path = local_provider_key_path(&config).unwrap();
        assert!(key_path.exists());
        let _ = fs::remove_dir_all(directory);
    }
}
