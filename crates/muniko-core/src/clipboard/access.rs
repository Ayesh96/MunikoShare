use std::borrow::Cow;

use super::content::ClipboardContent;
use super::png_codec;
use crate::error::{MunikoError, Result};
use crate::messages::clipboard as msg;

/// Seam over the OS clipboard so tests and callers can swap it; calls are blocking.
pub trait ClipboardAccess: Send + Sync {
    /// `None` when the clipboard is empty or holds neither text nor an image.
    fn read_content(&self) -> Result<Option<ClipboardContent>>;
    fn write_content(&self, content: &ClipboardContent) -> Result<()>;

    fn read_text(&self) -> Result<Option<String>> {
        Ok(match self.read_content()? {
            Some(ClipboardContent::Text(text)) => Some(text),
            _ => None,
        })
    }

    fn write_text(&self, text: &str) -> Result<()> {
        self.write_content(&ClipboardContent::Text(text.to_string()))
    }
}

/// The real system clipboard, opened per call so the type stays Send + Sync on every OS.
#[derive(Default)]
pub struct SystemClipboard;

impl SystemClipboard {
    pub fn new() -> Self {
        Self
    }

    fn open() -> Result<arboard::Clipboard> {
        arboard::Clipboard::new().map_err(|e| clipboard_error(msg::OPEN_FAILED, e))
    }

    fn read_image(clipboard: &mut arboard::Clipboard) -> Result<Option<ClipboardContent>> {
        match clipboard.get_image() {
            Ok(image) => {
                let (width, height) = (image.width as u32, image.height as u32);
                let png = png_codec::encode(width, height, &image.bytes)?;
                Ok(Some(ClipboardContent::Image { width, height, png }))
            }
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(e) => Err(clipboard_error(msg::READ_FAILED, e)),
        }
    }
}

impl ClipboardAccess for SystemClipboard {
    fn read_content(&self) -> Result<Option<ClipboardContent>> {
        let mut clipboard = Self::open()?;
        // Text wins: apps that copy text often add a rendered image too.
        match clipboard.get_text() {
            Ok(text) if !text.is_empty() => return Ok(Some(ClipboardContent::Text(text))),
            Ok(_) | Err(arboard::Error::ContentNotAvailable) => {}
            Err(e) => return Err(clipboard_error(msg::READ_FAILED, e)),
        }
        Self::read_image(&mut clipboard)
    }

    fn write_content(&self, content: &ClipboardContent) -> Result<()> {
        let mut clipboard = Self::open()?;
        let written = match content {
            ClipboardContent::Text(text) => clipboard.set_text(text.as_str()),
            ClipboardContent::Image { png, .. } => {
                let image = png_codec::decode(png)?;
                let data = arboard::ImageData { width: image.width as usize, height: image.height as usize, bytes: Cow::Owned(image.pixels) };
                clipboard.set_image(data)
            }
        };
        written.map_err(|e| clipboard_error(msg::WRITE_FAILED, e))
    }
}

fn clipboard_error(context: &str, detail: impl std::fmt::Display) -> MunikoError {
    MunikoError::Clipboard(format!("{context}: {detail}"))
}
