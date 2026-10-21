//! Turns an untrusted file name into a safe single path component.

use crate::constants::transfer::{
    FALLBACK_CHAR, INVALID_NAME_CHARS, MAX_FILE_NAME_CHARS, MAX_PATH_DEPTH, MAX_RELATIVE_PATH_CHARS, PATH_SEPARATOR, RESERVED_WINDOWS_NAMES,
};

/// Validates a `/`-separated relative path; rejects `..`, absolute, drive and empty parts instead of repairing them.
pub fn sanitize_relative_path(raw: &str) -> Option<String> {
    if raw.chars().count() > MAX_RELATIVE_PATH_CHARS {
        return None;
    }
    let parts: Vec<&str> = raw.split(PATH_SEPARATOR).collect();
    if parts.len() > MAX_PATH_DEPTH {
        return None;
    }
    let cleaned: Option<Vec<String>> = parts.into_iter().map(sanitize_component).collect();
    Some(cleaned?.join(&PATH_SEPARATOR.to_string()))
}

fn sanitize_component(part: &str) -> Option<String> {
    let mut chars = part.chars();
    let drive_letter = chars.next().is_some_and(|c| c.is_ascii_alphabetic()) && chars.next() == Some(':');
    if drive_letter || part.contains('\\') {
        return None;
    }
    sanitize_file_name(part)
}

/// Name for the nth duplicate folder: "Photos" -> "Photos (1)".
pub fn numbered_folder_name(name: &str, index: u32) -> String {
    format!("{name} ({index})")
}

/// Returns a safe file name, or None when nothing usable remains.
pub fn sanitize_file_name(raw: &str) -> Option<String> {
    let last_component = raw.rsplit(['/', '\\']).next().unwrap_or_default();
    let cleaned: String = last_component
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| if INVALID_NAME_CHARS.contains(&c) { FALLBACK_CHAR } else { c })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.').trim_end();
    if trimmed.is_empty() {
        return None;
    }
    let truncated: String = trimmed.chars().take(MAX_FILE_NAME_CHARS).collect();
    Some(avoid_reserved_name(truncated))
}

/// Name for the nth duplicate: "report.pdf" -> "report (1).pdf".
pub fn numbered_name(name: &str, index: u32) -> String {
    match name.rfind('.').filter(|dot| *dot > 0) {
        Some(dot) => format!("{} ({}){}", &name[..dot], index, &name[dot..]),
        None => format!("{name} ({index})"),
    }
}

fn avoid_reserved_name(name: String) -> String {
    let stem = name.split('.').next().unwrap_or_default().trim_end();
    let reserved = RESERVED_WINDOWS_NAMES.iter().any(|reserved| reserved.eq_ignore_ascii_case(stem));
    if reserved {
        format!("{FALLBACK_CHAR}{name}")
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_directories() {
        assert_eq!(sanitize_file_name("../../evil.txt").as_deref(), Some("evil.txt"));
        assert_eq!(sanitize_file_name("a\\b\\c.txt").as_deref(), Some("c.txt"));
        assert_eq!(sanitize_file_name("/etc/passwd").as_deref(), Some("passwd"));
    }

    #[test]
    fn rejects_empty_and_dot_names() {
        for raw in ["", "   ", ".", "..", "dir/", "../.."] {
            assert_eq!(sanitize_file_name(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn prefixes_reserved_windows_names() {
        assert_eq!(sanitize_file_name("CON").as_deref(), Some("_CON"));
        assert_eq!(sanitize_file_name("nul.txt").as_deref(), Some("_nul.txt"));
        assert_eq!(sanitize_file_name("console.txt").as_deref(), Some("console.txt"));
    }

    #[test]
    fn replaces_invalid_characters() {
        assert_eq!(sanitize_file_name("a:b?.txt").as_deref(), Some("a_b_.txt"));
    }

    #[test]
    fn relative_paths_keep_structure_and_reject_traversal() {
        assert_eq!(sanitize_relative_path("Photos/trip/b.jpg").as_deref(), Some("Photos/trip/b.jpg"));
        for raw in ["../x", "a/../b", "/etc/x", "C:\\x", "C:x", "a//b", "a/", "", "a\\b", "./a"] {
            assert_eq!(sanitize_relative_path(raw), None, "{raw:?}");
        }
        assert_eq!(sanitize_relative_path(&["a"; MAX_PATH_DEPTH + 1].join("/")), None);
    }

    #[test]
    fn numbers_duplicates() {
        assert_eq!(numbered_name("report.pdf", 1), "report (1).pdf");
        assert_eq!(numbered_name("archive.tar.gz", 2), "archive.tar (2).gz");
        assert_eq!(numbered_name("README", 3), "README (3)");
        assert_eq!(numbered_name(".env", 1), ".env (1)");
    }
}
