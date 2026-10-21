//! Registers the Windows "Send to" shortcut at startup; the NSIS hooks do the same at install time.

#[cfg(windows)]
pub fn register() {
    if let Err(err) = windows_send_to::create() {
        tracing::warn!(%err, "{}", crate::messages::logs::SEND_TO_LINK_FAILED);
    }
}

#[cfg(not(windows))]
pub fn register() {}

#[cfg(windows)]
mod windows_send_to {
    use std::path::PathBuf;

    use mslnk::ShellLink;

    use crate::constants::launch::SEND_FLAG;
    use crate::constants::windows::{SEND_TO_APPDATA_SUBPATH, SEND_TO_LINK_NAME};

    /// Rewritten every start so a moved install keeps working.
    pub fn create() -> std::io::Result<()> {
        let appdata = std::env::var_os("APPDATA").ok_or_else(|| std::io::Error::other("APPDATA is not set"))?;
        let dir = SEND_TO_APPDATA_SUBPATH.iter().fold(PathBuf::from(appdata), |path, part| path.join(part));
        std::fs::create_dir_all(&dir)?;
        let mut link = ShellLink::new(std::env::current_exe()?).map_err(std::io::Error::other)?;
        link.set_arguments(Some(SEND_FLAG.to_string()));
        link.create_lnk(dir.join(SEND_TO_LINK_NAME)).map_err(std::io::Error::other)
    }
}
