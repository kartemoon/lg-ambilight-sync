use crate::web::SharedState;
use std::{fs, path::Path, sync::Arc, time::Duration};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

const RESTART_MARKER: &str = "/run/lg-hue-sync.capture-recovery";
const RETRY_MARKER: &str = "/run/lg-hue-sync.capture-recovery-retry";
const RETRY_DELAY: Duration = Duration::from_secs(40);

pub fn reset_attempts() {
    let _ = fs::remove_file(RETRY_MARKER);
    let _ = fs::remove_file(RESTART_MARKER);
}

pub async fn supervise(shared: Arc<SharedState>, shutdown: CancellationToken) {
    // The service manager restarts this process. A manual CLI run must not exit here.
    if std::env::var_os("INVOCATION_ID").is_none() { return; }
    let marker = Path::new(RESTART_MARKER);
    let retry_marker = Path::new(RETRY_MARKER);
    let mut unhealthy = None;
    let mut healthy = None;
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => return,
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
        }
        let (deadline, running) = shared.capture_watch_sample();
        if running {
            let since = healthy.get_or_insert_with(Instant::now);
            if since.elapsed() >= Duration::from_secs(25) && marker.exists() {
                reset_attempts();
            }
        } else {
            healthy = None;
        }

        let Some(deadline) = deadline else {
            unhealthy = None;
            continue;
        };
        let (previous_deadline, since) = unhealthy.get_or_insert_with(|| (deadline, Instant::now()));
        if *previous_deadline != deadline {
            *previous_deadline = deadline;
            *since = Instant::now();
        }
        // A source transition can outlast the first restart. Give the driver longer
        // before the final attempt, and persist both attempts across process exits.
        let (attempt_marker, delay) = if marker.exists() {
            (retry_marker, deadline.max(RETRY_DELAY))
        } else {
            (marker, deadline)
        };
        if since.elapsed() < delay {
            continue;
        }
        if fs::OpenOptions::new().write(true).create_new(true).open(attempt_marker).is_ok() {
            tracing::warn!("Захват LG недоступен. Автоматически перезапускаю службу.");
            // A native driver call can block the main loop, including its cleanup.
            std::process::exit(75);
        }
    }
}
