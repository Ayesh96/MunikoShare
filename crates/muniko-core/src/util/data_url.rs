use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::constants::clipboard::PNG_DATA_URL_PREFIX;

/// A `data:image/png;base64,...` URL for direct use in an `<img>`.
pub fn png_data_url(png: &[u8]) -> String {
    format!("{PNG_DATA_URL_PREFIX}{}", STANDARD.encode(png))
}
