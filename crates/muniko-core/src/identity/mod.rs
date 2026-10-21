//! Device identity: a self-signed certificate whose fingerprint is the device ID.

use std::path::{Path, PathBuf};

use crate::constants::identity::{CERT_FILE_NAME, CERT_SUBJECT_ALT_NAME, KEY_FILE_NAME};
use crate::error::{MunikoError, Result};
use crate::messages::identity as messages;
use crate::util::fingerprint::fingerprint;
use crate::util::fs::ensure_dir;

#[derive(Debug, Clone)]
pub struct Identity {
    device_id: String,
    cert_der: Vec<u8>,
    key_der: Vec<u8>,
}

impl Identity {
    pub fn new(device_id: String, cert_der: Vec<u8>, key_der: Vec<u8>) -> Self {
        Self { device_id, cert_der, key_der }
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn cert_der(&self) -> &[u8] {
        &self.cert_der
    }

    pub fn key_der(&self) -> &[u8] {
        &self.key_der
    }
}

/// Loads the identity from disk, creating it on first run.
pub struct IdentityStore {
    dir: PathBuf,
}

impl IdentityStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub async fn load_or_create(&self) -> Result<Identity> {
        let cert_path = self.dir.join(CERT_FILE_NAME);
        let key_path = self.dir.join(KEY_FILE_NAME);
        let identity = match (cert_path.exists(), key_path.exists()) {
            (true, true) => Self::load(&cert_path, &key_path).await?,
            (false, false) => self.create(&cert_path, &key_path).await?,
            _ => return Err(MunikoError::Identity(messages::INCOMPLETE_FILES.to_string())),
        };
        Ok(identity)
    }

    async fn load(cert_path: &Path, key_path: &Path) -> Result<Identity> {
        let cert_der = tokio::fs::read(cert_path).await?;
        let key_der = tokio::fs::read(key_path).await?;
        tracing::info!("{}", messages::LOADED);
        Ok(Identity::new(fingerprint(&cert_der), cert_der, key_der))
    }

    async fn create(&self, cert_path: &Path, key_path: &Path) -> Result<Identity> {
        ensure_dir(&self.dir).await?;
        let generated = rcgen::generate_simple_self_signed(vec![CERT_SUBJECT_ALT_NAME.to_string()])
            .map_err(|error| MunikoError::Identity(format!("{}: {error}", messages::GENERATE_FAILED)))?;
        let cert_der = generated.cert.der().to_vec();
        let key_der = generated.signing_key.serialize_der();
        tokio::fs::write(cert_path, &cert_der).await?;
        write_private(key_path, &key_der).await?;
        tracing::info!("{}", messages::CREATED);
        Ok(Identity::new(fingerprint(&cert_der), cert_der, key_der))
    }
}

/// Writes the key file readable by the owner only on unix.
async fn write_private(path: &Path, contents: &[u8]) -> Result<()> {
    tokio::fs::write(path, contents).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = crate::constants::identity::KEY_FILE_MODE;
        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creates_files_then_reloads_same_identity() {
        let dir = tempfile::tempdir().unwrap();
        let store = IdentityStore::new(dir.path().to_path_buf());

        let first = store.load_or_create().await.unwrap();
        assert!(dir.path().join(CERT_FILE_NAME).exists());
        assert!(dir.path().join(KEY_FILE_NAME).exists());

        let second = store.load_or_create().await.unwrap();
        assert_eq!(first.device_id(), second.device_id());
        assert_eq!(first.cert_der(), second.cert_der());
        assert_eq!(first.device_id(), fingerprint(first.cert_der()));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn key_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        IdentityStore::new(dir.path().to_path_buf()).load_or_create().await.unwrap();
        let mode = std::fs::metadata(dir.path().join(KEY_FILE_NAME)).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
