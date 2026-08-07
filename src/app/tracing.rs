use std::{env, path::Path};

use anyhow::Result;
use axum::{
    body::{Body, to_bytes},
    http::{Request, Response},
    middleware::Next,
    response::IntoResponse,
};
use tokio::time::Instant;
use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, registry, util::SubscriberInitExt};

use crate::cfg::LogConfig;

fn stdout_layer<S>()
-> fmt::Layer<S, fmt::format::DefaultFields, fmt::format::Format, fn() -> std::io::Stdout>
where
    S: tracing::Subscriber,
    S: for<'span> registry::LookupSpan<'span>,
{
    fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_ansi(true)
}

fn file_layer<S>(
    exe_name: &str,
    log_dir: &Path,
) -> Result<(
    fmt::Layer<S, fmt::format::DefaultFields, fmt::format::Format, NonBlocking>,
    WorkerGuard,
)>
where
    S: tracing::Subscriber,
    S: for<'span> registry::LookupSpan<'span>,
{
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(exe_name)
        .filename_suffix("log")
        .build(log_dir)?;
    let (non_blocking_file, worker_guard) = tracing_appender::non_blocking(file_appender);

    Ok((
        fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .with_thread_ids(true)
            .with_writer(non_blocking_file)
            .with_ansi(false),
        worker_guard,
    ))
}

#[must_use]
pub fn init_tracing(log: &LogConfig, exe_name: &str) -> Result<Option<WorkerGuard>> {
    unsafe {
        env::set_var("RUST_BACKTRACE", "full");
    }

    let log_builder = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| log.level.as_str().into()))
        .with(stdout_layer());

    let mut file_worker = None;
    if let Some(log_dir) = &log.dir {
        let (file_layer, worker_guard) = file_layer(exe_name, log_dir)?;
        log_builder.with(file_layer).try_init()?;
        file_worker = Some(worker_guard);
    } else {
        log_builder.try_init()?;
    }

    Ok(file_worker)
}

pub async fn log_middleware(request: Request<Body>, next: Next) -> impl IntoResponse {
    let request = {
        let (parts, body) = request.into_parts();
        tracing::debug!("Request: {parts:#?}");
        let body = if let Ok(bytes) = to_bytes(body, usize::MAX).await {
            let body_str = String::from_utf8_lossy(&bytes);
            tracing::debug!("Request Body: {body_str}");
            Body::from(bytes)
        } else {
            Body::from(vec![])
        };
        Request::from_parts(parts, body)
    };

    let start = Instant::now();
    let response = next.run(request).await;

    let (parts, body) = response.into_parts();
    tracing::debug!(
        "run time: {}ms, Response: {parts:#?}",
        start.elapsed().as_millis(),
    );

    let body = if let Ok(bytes) = to_bytes(body, usize::MAX).await {
        let body_str = String::from_utf8_lossy(&bytes);
        tracing::debug!("Response Body: {body_str}");
        Body::from(bytes)
    } else {
        Body::from(vec![])
    };

    Response::from_parts(parts, body)
}
