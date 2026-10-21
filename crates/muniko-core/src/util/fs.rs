use std::path::{Path, PathBuf};

use tokio::fs::OpenOptions;

use super::sanitize::numbered_name;
use crate::error::Result;

/// Creates the directory and its parents if missing.
pub async fn ensure_dir(path: &Path) -> Result<()> {
    tokio::fs::create_dir_all(path).await?;
    Ok(())
}

/// Reserves a not-yet-existing file in `dir` ("a.txt", "a (1).txt", ...); create_new makes the claim atomic.
pub async fn claim_unique_path(dir: &Path, name: &str) -> std::io::Result<PathBuf> {
    let mut index = 0;
    loop {
        let candidate = if index == 0 { name.to_string() } else { numbered_name(name, index) };
        let path = dir.join(candidate);
        match OpenOptions::new().write(true).create_new(true).open(&path).await {
            Ok(_) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => index += 1,
            Err(error) => return Err(error),
        }
    }
}
