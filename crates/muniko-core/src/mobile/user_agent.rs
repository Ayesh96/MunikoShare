//! A short "Device - Browser" label from a User-Agent header.

use crate::constants::mobile::{LABEL_SEPARATOR, UNKNOWN_DEVICE_LABEL};

// Order matters: iPhone agents also contain "Mac OS", Chrome agents contain "Safari".
const DEVICES: [(&str, &str); 6] =
    [("iPhone", "iPhone"), ("iPad", "iPad"), ("Android", "Android"), ("Windows", "Windows"), ("Macintosh", "Mac"), ("Linux", "Linux")];
const BROWSERS: [(&str, &str); 6] =
    [("Edg", "Edge"), ("OPR", "Opera"), ("Firefox", "Firefox"), ("FxiOS", "Firefox"), ("CriOS", "Chrome"), ("Chrome", "Chrome")];

pub fn label_from_user_agent(agent: &str) -> String {
    let device = find(&DEVICES, agent).unwrap_or(UNKNOWN_DEVICE_LABEL);
    let browser = find(&BROWSERS, agent).or_else(|| agent.contains("Safari").then_some("Safari"));
    match browser {
        Some(browser) => format!("{device}{LABEL_SEPARATOR}{browser}"),
        None => device.to_string(),
    }
}

fn find(table: &[(&'static str, &'static str)], agent: &str) -> Option<&'static str> {
    table.iter().find(|(needle, _)| agent.contains(needle)).map(|(_, name)| *name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_common_agents() {
        let iphone = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Version/17.0 Mobile/15E148 Safari/604.1";
        assert_eq!(label_from_user_agent(iphone), "iPhone - Safari");
        let android = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 Chrome/120.0 Mobile Safari/537.36";
        assert_eq!(label_from_user_agent(android), "Android - Chrome");
        assert_eq!(label_from_user_agent(""), "Phone");
    }
}
