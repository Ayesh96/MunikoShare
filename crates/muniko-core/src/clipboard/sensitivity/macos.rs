use objc2::rc::autoreleasepool;
use objc2_app_kit::NSPasteboard;

use super::SensitivityProbe;
use crate::constants::clipboard::MACOS_SENSITIVE_TYPES;

/// Flags items carrying the nspasteboard.org concealed or transient markers.
#[derive(Default)]
pub struct PlatformProbe;

impl SensitivityProbe for PlatformProbe {
    fn is_sensitive(&self) -> bool {
        autoreleasepool(|_| {
            let Some(types) = NSPasteboard::generalPasteboard().types() else {
                return false;
            };
            types.to_vec().iter().any(|kind| MACOS_SENSITIVE_TYPES.contains(&kind.to_string().as_str()))
        })
    }
}
