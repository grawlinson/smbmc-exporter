//! HTTP endpoints.

use std::sync::Arc;
use std::time::Instant;

use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::{Registry, Unit};
use tokio::sync::Mutex;

use crate::collector::Snapshot;

/// Content type of the OpenMetrics text format.
pub const CONTENT_TYPE: &str = "application/openmetrics-text; version=1.0.0; charset=utf-8";

/// Buckets for `smbmc_collector_collect_seconds`.
const COLLECT_BUCKETS: [f64; 9] = [0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0];

const INDEX: &str = concat!(
    "<!doctype html><html><head><title>smbmc-exporter</title></head><body>",
    "<h1>smbmc-exporter ",
    env!("CARGO_PKG_VERSION"),
    "</h1><p><a href=\"metrics\">Metrics</a></p></body></html>"
);

/// State shared between requests.
#[derive(Debug)]
pub struct Exporter {
    client: smbmc::Client,
    collect_seconds: Histogram,
    /// Serialises scrapes: BMCs are slow and easily overwhelmed, so
    /// overlapping scrapes wait for each other rather than piling up.
    scrape: Mutex<()>,
}

impl Exporter {
    /// Wrap a BMC client.
    pub fn new(client: smbmc::Client) -> Self {
        Self {
            client,
            collect_seconds: Histogram::new(COLLECT_BUCKETS),
            scrape: Mutex::new(()),
        }
    }

    /// Fetch everything from the BMC. Errors are logged and leave the
    /// corresponding part of the snapshot empty.
    pub async fn snapshot(&self) -> Snapshot {
        let _guard = self.scrape.lock().await;
        let start = Instant::now();

        let power_supplies = self
            .client
            .pmbus_metrics()
            .await
            .inspect_err(|e| tracing::warn!("failed to fetch PMBus metrics: {e}"))
            .ok();
        let sensors = self
            .client
            .sensor_metrics()
            .await
            .inspect_err(|e| tracing::warn!("failed to fetch sensor metrics: {e}"))
            .ok();

        self.collect_seconds.observe(start.elapsed().as_secs_f64());
        Snapshot {
            sensors,
            power_supplies,
        }
    }

    /// Scrape the BMC and render the result in the OpenMetrics text format.
    pub async fn render(&self) -> Result<String, std::fmt::Error> {
        let snapshot = self.snapshot().await;

        let mut registry = Registry::default();
        registry.register_with_unit(
            "smbmc_collector_collect",
            "Time spent to collect metrics from SMBMC",
            Unit::Seconds,
            self.collect_seconds.clone(),
        );
        registry.register_collector(Box::new(snapshot));

        let mut body = String::new();
        encode(&mut body, &registry)?;
        Ok(body)
    }
}

/// Build the router: `/metrics` scrapes the BMC, `/` links to it.
pub fn router(exporter: Arc<Exporter>) -> Router {
    Router::new()
        .route("/", get(|| async { Html(INDEX) }))
        .route("/metrics", get(metrics))
        .with_state(exporter)
}

async fn metrics(State(exporter): State<Arc<Exporter>>) -> Response {
    match exporter.render().await {
        Ok(body) => ([(header::CONTENT_TYPE, CONTENT_TYPE)], body).into_response(),
        Err(e) => {
            tracing::error!("failed to encode metrics: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
