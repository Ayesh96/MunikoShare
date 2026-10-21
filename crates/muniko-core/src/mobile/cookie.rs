//! The session cookie: build it, clear it, read it back.

use axum::http::{header, HeaderMap};

use crate::constants::mobile::{COOKIE_ATTRIBUTES, SESSION_COOKIE};

pub fn session_cookie(session_id: &str, max_age_secs: u64) -> String {
    format!("{SESSION_COOKIE}={session_id}; {COOKIE_ATTRIBUTES}; Max-Age={max_age_secs}")
}

pub fn clear_cookie() -> String {
    session_cookie("", 0)
}

pub fn read_session_id(headers: &HeaderMap) -> Option<String> {
    let prefix = format!("{SESSION_COOKIE}=");
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|line| line.split(';'))
        .find_map(|pair| pair.trim().strip_prefix(&prefix).filter(|id| !id.is_empty()).map(str::to_string))
}
