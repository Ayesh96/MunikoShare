use std::future::Future;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// A spawned loop that stops when cancelled; the one place long-running loops are started and joined.
pub struct BackgroundTask {
    cancel: CancellationToken,
    handle: Option<JoinHandle<()>>,
}

impl BackgroundTask {
    pub fn spawn<F, Fut>(run: F) -> Self
    where
        F: FnOnce(CancellationToken) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let cancel = CancellationToken::new();
        let handle = tokio::spawn(run(cancel.clone()));
        Self { cancel, handle: Some(handle) }
    }

    pub async fn stop(&mut self) {
        self.cancel.cancel();
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}
