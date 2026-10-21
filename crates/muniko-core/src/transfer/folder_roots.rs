//! Picks one collision-free top-level folder per sent folder and reuses it for the whole transfer.

use std::collections::HashMap;
use std::path::PathBuf;

use tokio::fs;
use tokio::sync::Mutex;

use super::rejection::Rejection;
use crate::util::sanitize::numbered_folder_name;
use crate::messages::transfer as msg;

pub struct FolderRoots {
    download_dir: PathBuf,
    resolved: Mutex<HashMap<String, PathBuf>>,
}

impl FolderRoots {
    pub fn new(download_dir: PathBuf) -> Self {
        Self { download_dir, resolved: Mutex::new(HashMap::new()) }
    }

    /// Creates the folder (`Photos`, else `Photos (1)`, ...) on first use; create_dir is the atomic claim.
    pub async fn resolve(&self, top_name: &str) -> Result<PathBuf, Rejection> {
        let mut resolved = self.resolved.lock().await;
        if let Some(path) = resolved.get(top_name) {
            return Ok(path.clone());
        }
        let mut index = 0;
        loop {
            let name = if index == 0 { top_name.to_string() } else { numbered_folder_name(top_name, index) };
            let path = self.download_dir.join(name);
            match fs::create_dir(&path).await {
                Ok(()) => {
                    resolved.insert(top_name.to_string(), path.clone());
                    return Ok(path);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => index += 1,
                Err(_) => return Err(Rejection::Internal(msg::STORE_FAILED)),
            }
        }
    }
}
