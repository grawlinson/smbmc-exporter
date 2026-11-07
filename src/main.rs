//! `smbmc-exporter` binary.

use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use smbmc_exporter::{Args, Exporter, router};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    match run(Args::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let client = args.client()?;
    tracing::info!("Connecting to {}", args.server_url());

    let listener = TcpListener::bind((args.listen_addr.as_str(), args.listen_port))
        .await
        .map_err(|e| {
            format!(
                "cannot listen on {}:{}: {e}",
                args.listen_addr, args.listen_port
            )
        })?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, router(Arc::new(Exporter::new(client))))
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

/// Resolves on Ctrl-C or, on Unix, SIGTERM.
async fn shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
    tracing::info!("Shutting down");
}
