pub use kid_app::server::ssr::SharedTaskCache;
use kid_types::server::FlushError;

use tokio::time::{self, Duration, Instant};
use tokio_util::sync::CancellationToken;

pub trait TaskCacheFlush<'a>: Default {
    const FLUSH_INTERVAL: Duration;
    const FLUSH_TIMEOUT: Duration;

    async fn background_flush(&self, shutdown: CancellationToken);
    async fn final_flush(&self);
}

impl<'a> TaskCacheFlush<'a> for SharedTaskCache {
    const FLUSH_INTERVAL: Duration = Duration::from_mins(1);
    const FLUSH_TIMEOUT: Duration = Duration::from_secs(4);

    async fn background_flush(&self, shutdown: CancellationToken) {
        let mut interval = time::interval(Self::FLUSH_INTERVAL);
        let mut flush_failed_count = 0;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    tracing::trace!("flush task cache..");
                    let mut cache = self.write().await;
                    match cache.flush().await {
                        Ok(num) => {
                            if num > 0 {
                                tracing::debug!("flush task cache.. {num} tasks successfully flushed.");
                            } else {
                                tracing::trace!("flush task cache.. done.");
                            }
                            flush_failed_count = 0;
                        }
                        Err(e) => {
                            tracing::warn!("flush task cache.. with errors: {e}");
                            match e {
                                FlushError::ErrorList(failed, _, _) => {
                                    interval.reset_after(Self::FLUSH_INTERVAL / (failed + 1) as u32);
                                }
                                _ => {
                                    interval.reset_after(Self::FLUSH_INTERVAL / 2);
                                }
                            }

                            flush_failed_count += 1;
                            if flush_failed_count > 10 {
                                // reset to prevent error spamming
                                flush_failed_count = 0;
                                // we want a detailed error report (with suberrors)
                                let e = miette::Report::from(e);
                                tracing::error!("failed to flush task cache:\n{e:?}")
                            }
                        }
                    }
                }
                _ = shutdown.cancelled() => {
                    tracing::info!("Background task flush shutting down");
                    break;
                }
            }
        }
    }

    async fn final_flush(&self) {
        tracing::trace!("flush task cache finally..");
        let mut last_error: Option<FlushError> = None;
        let start = Instant::now();
        let mut cache = self.write().await;
        while start.elapsed() < Self::FLUSH_TIMEOUT {
            match cache.flush().await {
                Ok(num) => {
                    tracing::info!("flush task cache.. {num} tasks successfully flushed.");
                    return;
                }
                Err(e) => {
                    tracing::warn!("flush task cache.. with errors: {e}");
                    last_error = Some(e);
                    // give storage IO some time to relax..
                    time::sleep(Duration::from_millis(10)).await;
                }
            }
        }

        if let Some(e) = last_error {
            // we want a detailed error report (with suberrors)
            let e = miette::Report::from(e);
            tracing::error!(
                "flush task cache.. timeout after {:?} with error:\n{e:?}",
                Self::FLUSH_TIMEOUT,
            )
        } else {
            tracing::info!("flush task cache.. successfully completed");
        }
    }
}
