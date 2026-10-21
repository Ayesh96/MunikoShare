use std::io::Cursor;

use crate::constants::clipboard::MAX_IMAGE_DECODED_BYTES;
use crate::error::{MunikoError, Result};
use crate::messages::clipboard as msg;

/// Straight RGBA8 pixels, the layout the OS clipboard APIs use.
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Encodes RGBA8 pixels as PNG with fast compression to keep polling cheap.
pub fn encode(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder.write_header().map_err(|e| encode_error(e))?;
    writer.write_image_data(rgba).map_err(|e| encode_error(e))?;
    writer.finish().map_err(|e| encode_error(e))?;
    Ok(out)
}

/// Reads only the PNG header; rejects images that would decode too large.
pub fn dimensions(png: &[u8]) -> Result<(u32, u32)> {
    let reader = png::Decoder::new(Cursor::new(png)).read_info().map_err(|e| decode_error(e))?;
    let info = reader.info();
    let decoded = info.width as usize * info.height as usize * 4;
    if decoded > MAX_IMAGE_DECODED_BYTES {
        return Err(MunikoError::Clipboard(msg::IMAGE_TOO_LARGE.to_string()));
    }
    Ok((info.width, info.height))
}

/// Decodes any PNG color type into RGBA8.
pub fn decode(png: &[u8]) -> Result<RgbaImage> {
    dimensions(png)?;
    let mut decoder = png::Decoder::new(Cursor::new(png));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| decode_error(e))?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or_else(|| MunikoError::Clipboard(msg::IMAGE_TOO_LARGE.to_string()))?];
    let frame = reader.next_frame(&mut buffer).map_err(|e| decode_error(e))?;
    buffer.truncate(frame.buffer_size());
    let pixels = to_rgba(frame.color_type, buffer)?;
    Ok(RgbaImage { width: frame.width, height: frame.height, pixels })
}

fn to_rgba(color: png::ColorType, data: Vec<u8>) -> Result<Vec<u8>> {
    Ok(match color {
        png::ColorType::Rgba => data,
        png::ColorType::Rgb => data.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => data.chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Grayscale => data.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return Err(MunikoError::Clipboard(msg::IMAGE_INVALID.to_string())),
    })
}

fn encode_error(error: impl std::fmt::Display) -> MunikoError {
    MunikoError::Clipboard(format!("{}: {error}", msg::IMAGE_ENCODE_FAILED))
}

fn decode_error(error: impl std::fmt::Display) -> MunikoError {
    MunikoError::Clipboard(format!("{}: {error}", msg::IMAGE_DECODE_FAILED))
}
