use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::constants::io::READ_BUFFER_BYTES;

/// Streams up to `limit` bytes into a hasher; returns it with the byte count actually read.
pub async fn hash_up_to<R: AsyncRead + Unpin>(reader: &mut R, limit: u64) -> std::io::Result<(Sha256, u64)> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; READ_BUFFER_BYTES];
    let mut total = 0u64;
    while total < limit {
        let want = (limit - total).min(buffer.len() as u64) as usize;
        let read = reader.read(&mut buffer[..want]).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        total += read as u64;
    }
    Ok((hasher, total))
}

/// Hex SHA-256 and size of a whole reader, streamed.
pub async fn hash_all<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<(String, u64)> {
    let (hasher, size) = hash_up_to(reader, u64::MAX).await?;
    Ok((hex::encode(hasher.finalize()), size))
}

/// Copies a reader into a writer while hashing; returns the hex SHA-256 and the byte count.
pub async fn copy_hashed<R, W>(reader: &mut R, writer: &mut W) -> std::io::Result<(String, u64)>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; READ_BUFFER_BYTES];
    let mut total = 0u64;
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        writer.write_all(&buffer[..read]).await?;
        total += read as u64;
    }
    writer.flush().await?;
    Ok((hex::encode(hasher.finalize()), total))
}

/// True for a lowercase hex SHA-256 digest; the only shape allowed in cache file names.
pub fn is_sha256_hex(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
