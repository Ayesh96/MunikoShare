//! Turns command-line arguments and file URLs into existing absolute paths.

use std::path::{Path, PathBuf};

use url::Url;

use crate::constants::launch::{FILE_URL_SCHEME, SEND_FLAG};

/// Skips the executable name and flags; relative paths resolve against `cwd`.
pub fn paths_from_args(args: &[String], cwd: &Path) -> Vec<String> {
    args.iter()
        .skip(1)
        .filter(|arg| arg.as_str() != SEND_FLAG && !arg.starts_with("--"))
        .map(|arg| cwd.join(arg))
        .filter_map(existing_path)
        .collect()
}

pub fn paths_from_urls(urls: &[Url]) -> Vec<String> {
    urls.iter()
        .filter(|url| url.scheme() == FILE_URL_SCHEME)
        .filter_map(|url| url.to_file_path().ok())
        .filter_map(existing_path)
        .collect()
}

fn existing_path(path: PathBuf) -> Option<String> {
    path.exists().then(|| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_existing_paths_and_skips_flags() {
        let dir = std::env::temp_dir();
        let args = vec!["app".to_string(), SEND_FLAG.to_string(), "--x".to_string(), "missing-file-xyz".to_string(), dir.to_string_lossy().into_owned()];
        assert_eq!(paths_from_args(&args, &dir), vec![dir.to_string_lossy().into_owned()]);
    }
}
