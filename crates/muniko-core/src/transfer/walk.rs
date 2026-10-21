//! Expands the paths the user picked into the files to send, keeping folder structure.

use std::path::{Path, PathBuf};

use tokio::fs;

use crate::constants::transfer::{MAX_FILES_PER_OFFER, PATH_SEPARATOR};
use crate::error::{MunikoError, Result};
use crate::messages::transfer as msg;

/// One file to send and the `/`-separated path the receiver sees it under.
pub struct SendItem {
    pub path: PathBuf,
    pub relative: String,
}

fn read_error(error: std::io::Error) -> MunikoError {
    MunikoError::Transfer(format!("{}: {error}", msg::READ_FILE_FAILED))
}

/// Files stay as they are; folders are walked recursively, skipping symlinks.
pub async fn collect_items(paths: &[PathBuf]) -> Result<Vec<SendItem>> {
    let mut items = Vec::new();
    for path in paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .ok_or_else(|| MunikoError::Transfer(msg::NOT_A_FILE.to_string()))?;
        if fs::metadata(path).await.map_err(read_error)?.is_dir() {
            walk_dir(path, name, &mut items).await?;
        } else {
            items.push(SendItem { path: path.clone(), relative: name });
        }
    }
    if items.is_empty() {
        return Err(MunikoError::Transfer(msg::NO_FILES_TO_SEND.to_string()));
    }
    Ok(items)
}

async fn walk_dir(root: &Path, root_name: String, items: &mut Vec<SendItem>) -> Result<()> {
    let mut pending = vec![(root.to_path_buf(), root_name)];
    while let Some((dir, relative_dir)) = pending.pop() {
        for (name, path, file_type) in read_sorted(&dir).await? {
            let relative = format!("{relative_dir}{PATH_SEPARATOR}{name}");
            if file_type.is_dir() {
                pending.push((path, relative));
            } else if file_type.is_file() {
                items.push(SendItem { path, relative });
            }
            if items.len() > MAX_FILES_PER_OFFER {
                return Err(MunikoError::Transfer(msg::TOO_MANY_FILES.to_string()));
            }
        }
    }
    Ok(())
}

// file_type() does not follow symlinks, so links are neither dirs nor files and get skipped.
async fn read_sorted(dir: &Path) -> Result<Vec<(String, PathBuf, std::fs::FileType)>> {
    let mut entries = Vec::new();
    let mut reader = fs::read_dir(dir).await.map_err(read_error)?;
    while let Some(entry) = reader.next_entry().await.map_err(read_error)? {
        let file_type = entry.file_type().await.map_err(read_error)?;
        entries.push((entry.file_name().to_string_lossy().into_owned(), entry.path(), file_type));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}
