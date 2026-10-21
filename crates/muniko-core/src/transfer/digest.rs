use std::path::Path;

use tokio::fs::File;

use super::manifest::ManifestFile;
use crate::error::{MunikoError, Result};
use crate::messages::transfer as msg;
use crate::util::hash::hash_all;

/// Reads a regular file once, streaming, to build its manifest entry under `relative_path`.
pub async fn describe_file(path: &Path, relative_path: String) -> Result<ManifestFile> {
    let read_error = |e: std::io::Error| MunikoError::Transfer(format!("{}: {e}", msg::READ_FILE_FAILED));
    let metadata = tokio::fs::metadata(path).await.map_err(read_error)?;
    if !metadata.is_file() {
        return Err(MunikoError::Transfer(msg::NOT_A_FILE.to_string()));
    }
    let mut file = File::open(path).await.map_err(read_error)?;
    let (sha256, size) = hash_all(&mut file).await.map_err(read_error)?;
    Ok(ManifestFile { path: relative_path, size, sha256 })
}
