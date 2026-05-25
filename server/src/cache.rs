pub use kid_app::server::ssr::SharedEventBus;
pub use kid_app::server::ssr::SharedTaskCache;
pub use kid_app::server::ssr::SharedTimeOffset;

use kid_app::events::{FlushOutcome, ServerEvent};

use tokio::time::{self, Duration, Instant};
use tokio_util::sync::CancellationToken;

pub trait TaskCacheFlush<'a>: Default {
    const FLUSH_INTERVAL: Duration;
    const FLUSH_TIMEOUT: Duration;

    async fn background_flush(&self, shutdown: CancellationToken, events: &SharedEventBus);
    async fn final_flush(&self);
}

impl<'a> TaskCacheFlush<'a> for SharedTaskCache {
    const FLUSH_INTERVAL: Duration = Duration::from_mins(1);
    const FLUSH_TIMEOUT: Duration = Duration::from_secs(4);

    async fn background_flush(&self, shutdown: CancellationToken, events: &SharedEventBus) {
        const RETRY: usize = 10;
        let mut failed = 0;
        let mut interval = time::interval(Self::FLUSH_INTERVAL);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    tracing::trace!("flush task cache in background");
                    let mut cache = self.write().await;
                    match cache.flush().await {
                        Ok(num) => {
                            if num > 0 {
                                tracing::info!("{num} tasks successfully flushed");
                                let _ = events.send(ServerEvent::Flush(
                                    FlushOutcome::Success { count: num },
                                ));
                            } else {
                                tracing::debug!("no tasks to flush");
                            }
                            failed = 0;
                        }
                        Err(e) => {
                            tracing::warn!("{e}");
                            let _ = events.send(ServerEvent::Flush(
                                FlushOutcome::Error { message: e.to_string() },
                            ));
                            interval.reset_after(Self::FLUSH_INTERVAL / (e.failed() + 1) as u32);
                            failed += 1;
                            if failed > RETRY {
                                // reset to prevent error spamming
                                failed = 0;
                                // we want a detailed error report (with suberrors)
                                let e = miette::Report::from(e);
                                tracing::error!("failed to flush task cache {RETRY}-times:\n{e:?}")
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
        let mut last_error: Option<_> = None;
        let start = Instant::now();
        let mut cache = self.write().await;
        while start.elapsed() < Self::FLUSH_TIMEOUT {
            match cache.flush().await {
                Ok(num) => {
                    if num > 0 {
                        tracing::info!("{num} tasks successfully flushed");
                    } else {
                        tracing::info!("no tasks need to be flushed");
                    }
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
