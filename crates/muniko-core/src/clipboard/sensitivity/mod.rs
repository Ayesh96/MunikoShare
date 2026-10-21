//! Detects clipboard items that apps mark as secret (password managers) so they never sync.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod other;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub use macos::PlatformProbe;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use other::PlatformProbe;
#[cfg(target_os = "windows")]
pub use windows::PlatformProbe;

/// Asks the OS whether the current clipboard item is marked sensitive; blocking.
pub trait SensitivityProbe: Send + Sync {
    fn is_sensitive(&self) -> bool;
}
