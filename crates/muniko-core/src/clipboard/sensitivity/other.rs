use super::SensitivityProbe;

/// Linux has no standard marker that arboard exposes, so nothing is flagged.
/// Password managers there clear the clipboard after a timeout instead.
#[derive(Default)]
pub struct PlatformProbe;

impl SensitivityProbe for PlatformProbe {
    fn is_sensitive(&self) -> bool {
        false
    }
}
