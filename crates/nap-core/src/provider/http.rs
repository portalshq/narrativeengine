//! Shared Lore HTTP endpoint selection and provider configuration migration.
use anyhow::{Context, Result, bail};
use reqwest::Url;
use std::io::Write;
use std::path::Path;

pub fn validate_origin(value: &str) -> Result<Url> {
    let url = Url::parse(value).context("invalid Lore HTTP origin")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/")
    {
        bail!(
            "Lore HTTP URL must be an http(s) origin without credentials, path, query, or fragment"
        );
    }
    Ok(url)
}

/// Standard Lore deployments expose HTTP at 41339, or share the TLS edge at 443.
/// Operators can persist a custom origin in provider.toml; clients need no flags.
pub fn default_origin(remote: &str) -> Result<String> {
    if remote.is_empty() {
        return Ok("http://127.0.0.1:41339".into());
    }
    let rpc = Url::parse(remote).context("invalid Lore remote URL")?;
    if !rpc.username().is_empty()
        || rpc.password().is_some()
        || rpc.query().is_some()
        || rpc.fragment().is_some()
    {
        bail!("Lore remote URL must not contain credentials, query, or fragment");
    }
    let secure = match rpc.scheme() {
        "lore" | "grpc" | "http" => false,
        "lores" | "grpcs" | "https" => true,
        _ => bail!("unsupported Lore remote scheme"),
    };
    let host = rpc.host().context("Lore remote URL has no host")?;
    let cloud = rpc.host_str() == Some("lore.portals.works");
    let edge = cloud || (secure && matches!(rpc.port(), None | Some(443)));
    let scheme = if secure || cloud { "https" } else { "http" };
    let port = if edge { "" } else { ":41339" };
    Ok(format!("{scheme}://{host}{port}"))
}

fn same_server(a: &str, b: &str) -> bool {
    fn identity(value: &str) -> Option<(bool, String, u16)> {
        let url = Url::parse(value).ok()?;
        let secure = match url.scheme() {
            "lore" | "grpc" | "http" => false,
            "lores" | "grpcs" | "https" => true,
            _ => return None,
        };
        Some((
            secure,
            url.host_str()?.into(),
            url.port().unwrap_or(if secure { 443 } else { 41337 }),
        ))
    }
    match (identity(a), identity(b)) {
        (Some(a), Some(b)) => a == b,
        _ => a == b,
    }
}

/// Use configuration only for its own server, never for a repository on another host.
/// Backfill existing provider files once, preserving custom fields and HTTP origins.
pub fn configured_origin(nap_home: &Path, remote: &str) -> Result<String> {
    let path = nap_home.join("provider.toml");
    let mut config: toml::Value = match std::fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).context("invalid provider.toml")?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return default_origin(remote),
        Err(e) => return Err(e.into()),
    };
    let configured_remote = config
        .get("remote_url")
        .and_then(toml::Value::as_str)
        .unwrap_or_else(
            || match config.get("provider_type").and_then(toml::Value::as_str) {
                Some("local") => "lore://localhost:41337",
                Some("portals-cloud") => super::portals_cloud::PORTALS_CLOUD_URL,
                _ => "",
            },
        );
    if !same_server(configured_remote, remote) {
        return default_origin(remote);
    }
    if let Some(value) = config.get("http_url").and_then(toml::Value::as_str) {
        validate_origin(value)?;
        return Ok(value.trim_end_matches('/').into());
    }
    let origin = default_origin(remote)?;
    config
        .as_table_mut()
        .context("provider config must be a table")?
        .insert("http_url".into(), toml::Value::String(origin.clone()));
    let mut file = tempfile::NamedTempFile::new_in(nap_home)?;
    file.write_all(toml::to_string_pretty(&config)?.as_bytes())?;
    file.persist(path)
        .context("failed to save Lore HTTP endpoint")?;
    Ok(origin)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_local_and_cloud_configs_receive_http_origin() {
        for (provider, remote, origin) in [
            (
                "local",
                "grpc://localhost:41337/repo",
                "http://localhost:41339",
            ),
            (
                "portals-cloud",
                "grpcs://lore.portals.works/repo",
                "https://lore.portals.works",
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("provider.toml");
            std::fs::write(&path, format!("provider_type = {provider:?}\n")).unwrap();
            assert_eq!(configured_origin(dir.path(), remote).unwrap(), origin);
            let config: toml::Value =
                toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            assert_eq!(config["http_url"].as_str(), Some(origin));
        }
    }
    #[test]
    fn origins_cover_supported_transports_and_ipv6() {
        for (remote, expected) in [
            ("", "http://127.0.0.1:41339"),
            (
                "lore://100.105.14.118:41337/repo",
                "http://100.105.14.118:41339",
            ),
            ("grpc://[::1]:41337", "http://[::1]:41339"),
            ("lores://example.com:41337", "https://example.com:41339"),
            ("grpcs://example.com", "https://example.com"),
            (
                "grpcs://lore.portals.works/repo",
                "https://lore.portals.works",
            ),
            (
                "lore://lore.portals.works.attacker.test:41337",
                "http://lore.portals.works.attacker.test:41339",
            ),
        ] {
            assert_eq!(default_origin(remote).unwrap(), expected);
        }
        assert!(default_origin("lore://user:secret@host").is_err());
        assert!(validate_origin("https://host/path").is_err());
    }
    #[test]
    fn migration_is_idempotent_and_does_not_cross_servers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("provider.toml");
        std::fs::write(&path, "provider_type = \"remote\"\nremote_url = \"lore://host:41337\"\nworkspace_id = \"default\"\ncustom = 1\n").unwrap();
        assert_eq!(
            configured_origin(dir.path(), "lore://host:41337/repo").unwrap(),
            "http://host:41339"
        );
        let first = std::fs::read_to_string(&path).unwrap();
        configured_origin(dir.path(), "lore://host:41337").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), first);
        assert!(first.contains("custom = 1"));
        assert_eq!(
            configured_origin(dir.path(), "lore://other:41337").unwrap(),
            "http://other:41339"
        );
        let custom = first.replace("http://host:41339", "https://downloads.example.com");
        std::fs::write(&path, custom).unwrap();
        assert_eq!(
            configured_origin(dir.path(), "lore://host:41337").unwrap(),
            "https://downloads.example.com"
        );
    }
}
