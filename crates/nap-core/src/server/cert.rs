// SPDX-FileCopyrightText: 2026 Digital Creations
// SPDX-License-Identifier: MIT
//! Certificate generation for Lore QUIC server
//!
//! Uses rcgen to generate self-signed certificates compatible with Lore's QUIC endpoint.

use anyhow::{Context, Result};
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use std::fs;
use std::path::Path;

/// Generate self-signed certificates for Lore QUIC server
///
/// Creates a certificate and private key pair suitable for Lore's QUIC endpoint.
/// Certificates are persisted across restarts and only regenerated if missing or invalid.
pub fn generate_certificates(cert_dir: &Path) -> Result<CertificateFiles> {
    fs::create_dir_all(cert_dir)
        .context("Failed to create certificate directory")?;

    let cert_path = cert_dir.join("cert.pem");
    let key_path = cert_dir.join("key.pem");

    // Only regenerate if missing
    if cert_path.exists() && key_path.exists() {
        tracing::info!("Certificates already exist at {:?}", cert_dir);
        return Ok(CertificateFiles {
            cert_path,
            key_path,
        });
    }

    tracing::info!("Generating self-signed certificates for Lore QUIC");

    let mut params = CertificateParams::default();
    
    // Set distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, "localhost");
    dn.push(rcgen::DnType::OrganizationName, "NAP SDK");
    params.distinguished_name = dn;

    // Set subject alternative names for localhost
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName(rcgen::Ia5String::try_from("localhost").unwrap()),
        rcgen::SanType::IpAddress("127.0.0.1".parse().unwrap()),
        rcgen::SanType::IpAddress("::1".parse().unwrap()),
    ];

    // Generate certificate and key
    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    // Write certificate
    let cert_pem = cert.pem();
    fs::write(&cert_path, cert_pem)
        .context("Failed to write certificate file")?;

    // Write private key
    let key_pem = key_pair.serialize_pem();
    fs::write(&key_path, key_pem)
        .context("Failed to write private key file")?;

    // Set restrictive permissions on private key
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&key_path)
            .with_context(|| format!("Failed to read key file permissions at {}", key_path.display()))?
            .permissions();
        perms.set_mode(0o600); // owner read/write only
        fs::set_permissions(&key_path, perms)
            .with_context(|| format!("Failed to set restrictive permissions on key file at {}", key_path.display()))?;
        tracing::debug!("Set key file permissions to 0600 (owner-only)");
    }

    #[cfg(windows)]
    {
        // On Windows, mark the key file as hidden and system to discourage
        // casual access. NTFS ACLs provide stricter protection if needed.
        use windows_sys::Win32::Storage::FileSystem::{SetFileAttributesW, FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_SYSTEM};

        let key_wide: Vec<u16> = key_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        unsafe {
            let result = SetFileAttributesW(
                key_wide.as_ptr(),
                FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM,
            );
            if result == 0 {
                tracing::warn!(
                    "Failed to set hidden/system attributes on key file at {}. \
                     The key file may be visible in Explorer.",
                    key_path.display()
                );
            } else {
                tracing::debug!("Set key file attributes to hidden+system (Windows)");
            }
        }
    }

    tracing::info!("Certificates generated successfully at {:?}", cert_dir);

    Ok(CertificateFiles {
        cert_path,
        key_path,
    })
}

/// Paths to generated certificate files
#[derive(Debug, Clone)]
pub struct CertificateFiles {
    pub cert_path: std::path::PathBuf,
    pub key_path: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_certificates() {
        let temp_dir = TempDir::new().unwrap();
        let cert_dir = temp_dir.path();

        let files = generate_certificates(cert_dir).unwrap();

        assert!(files.cert_path.exists());
        assert!(files.key_path.exists());

        // Verify we can regenerate without error (should skip if exists)
        let files2 = generate_certificates(cert_dir).unwrap();
        assert_eq!(files.cert_path, files2.cert_path);
        assert_eq!(files.key_path, files2.key_path);
    }
}
