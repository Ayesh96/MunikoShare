use clipboard_win::raw::{is_format_avail, register_format};

use super::SensitivityProbe;
use crate::constants::clipboard::WINDOWS_EXCLUDE_FORMAT;

/// Flags items with the `ExcludeClipboardContentFromMonitorProcessing` format.
/// `CanIncludeInClipboardHistory` = 0 is not checked: reading it needs the clipboard open.
#[derive(Default)]
pub struct PlatformProbe;

impl SensitivityProbe for PlatformProbe {
    fn is_sensitive(&self) -> bool {
        register_format(WINDOWS_EXCLUDE_FORMAT).map(|format| is_format_avail(format.get())).unwrap_or(false)
    }
}
