//! Uploads one file to the peer and resumes from the peer's byte count after an interruption.

use std::io::SeekFrom;
use std::path::Path;
use std::sync::{Arc, Mutex};

use futures_util::TryStreamExt;
use reqwest::{header, Body, Client, RequestBuilder, Response, StatusCode};
use tokio::fs::File;
use tokio::io::AsyncSeekExt;
use tokio_util::io::ReaderStream;

use super::manifest::ReceivedStatus;
use super::peer_reply::{ensure_accepted, into_transfer_error, transfer_error};
use super::progress::ProgressReporter;
use crate::constants::io::READ_BUFFER_BYTES;
use crate::constants::transfer::*;
use crate::error::{MunikoError, Result};
use crate::messages::transfer as msg;
use crate::net::PeerClient;

pub type SharedProgress = Arc<Mutex<ProgressReporter>>;

// Retry means the next resume attempt may succeed; Fatal ends the upload.
enum Failure {
    Retry(MunikoError),
    Fatal(MunikoError),
}

pub struct FileUpload<'a> {
    pub client: &'a PeerClient,
    pub transfer_id: &'a str,
    pub index: usize,
    pub path: &'a Path,
    pub size: u64,
    /// Bytes of the transfer's earlier files, so progress continues from the right place.
    pub base_bytes: u64,
    pub progress: &'a SharedProgress,
}

impl FileUpload<'_> {
    pub async fn run(&self) -> Result<()> {
        let mut last_error = None;
        for attempt in 0..=RESUME_ATTEMPTS {
            let offset = if attempt == 0 {
                0
            } else {
                tracing::warn!(attempt, "{}", msg::UPLOAD_RESUMING);
                tokio::time::sleep(RESUME_BACKOFF * attempt).await;
                match self.query_received().await {
                    Ok(status) if status.complete => return Ok(()),
                    Ok(status) => status.received,
                    Err(Failure::Retry(error)) => {
                        last_error = Some(error);
                        continue;
                    }
                    Err(Failure::Fatal(error)) => return Err(error),
                }
            };
            self.set_progress(offset);
            match self.put_from(offset).await {
                Ok(()) => return Ok(()),
                Err(Failure::Retry(error)) => last_error = Some(error),
                Err(Failure::Fatal(error)) => return Err(error),
            }
        }
        Err(last_error.unwrap_or_else(|| MunikoError::Transfer(msg::SEND_FAILED.to_string())))
    }

    fn file_route(&self) -> String {
        format!("{ROUTE_TRANSFERS}/{}/files/{}", self.transfer_id, self.index)
    }

    fn set_progress(&self, offset: u64) {
        if let Ok(mut reporter) = self.progress.lock() {
            reporter.seek(self.base_bytes + offset);
        }
    }

    async fn query_received(&self) -> std::result::Result<ReceivedStatus, Failure> {
        let route = self.file_route();
        let response = self
            .client
            .send(|http, base_url| {
                let url = format!("{base_url}{route}");
                async move { Ok(http.get(url)) }
            })
            .await
            .map_err(classify_send_error)?;
        let response = classify_reply(response).await?;
        let bytes = response.bytes().await.map_err(|e| Failure::Retry(transfer_error(msg::BAD_RECEIVED_RESPONSE, e)))?;
        serde_json::from_slice(&bytes).map_err(|e| Failure::Fatal(transfer_error(msg::BAD_RECEIVED_RESPONSE, e)))
    }

    // Reopens the file per attempt so a retry on the next address streams from the same offset.
    async fn put_from(&self, offset: u64) -> std::result::Result<(), Failure> {
        let route = self.file_route();
        let response = self
            .client
            .send(|http, base_url| self.put_request(http, format!("{base_url}{route}"), offset))
            .await
            .map_err(classify_send_error)?;
        classify_reply(response).await.map(|_| ())
    }

    async fn put_request(&self, http: Client, url: String, offset: u64) -> Result<RequestBuilder> {
        let read_error = |e: std::io::Error| transfer_error(msg::READ_FILE_FAILED, e);
        let mut file = File::open(self.path).await.map_err(read_error)?;
        file.seek(SeekFrom::Start(offset)).await.map_err(read_error)?;
        let progress = self.progress.clone();
        let stream = ReaderStream::with_capacity(file, READ_BUFFER_BYTES).inspect_ok(move |chunk| {
            if let Ok(mut reporter) = progress.lock() {
                reporter.add(chunk.len() as u64);
            }
        });
        Ok(http
            .put(format!("{url}?{OFFSET_PARAM}={offset}"))
            .header(header::CONTENT_LENGTH, self.size.saturating_sub(offset))
            .body(Body::wrap_stream(stream)))
    }
}

fn classify_send_error(error: MunikoError) -> Failure {
    match error {
        MunikoError::Network(_) => Failure::Retry(into_transfer_error(error)),
        other => Failure::Fatal(other),
    }
}

// 409 can mean the server still holds the dropped upload, so it is worth another attempt.
async fn classify_reply(response: Response) -> std::result::Result<Response, Failure> {
    let busy = response.status() == StatusCode::CONFLICT;
    ensure_accepted(response).await.map_err(|error| if busy { Failure::Retry(error) } else { Failure::Fatal(error) })
}
