use anyhow::Context;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::net::SocketAddr;

pub fn init_metrics(metrics_addr: &str) -> anyhow::Result<PrometheusHandle> {
    let addr: SocketAddr = metrics_addr.parse().context("invalid metrics addr")?;

    let builder = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("request_latency_seconds".to_string()),
            &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5],
        )
        .context("failed to set metric buckets")?
        .with_http_listener(addr);

    let handle = builder.install_recorder().context("metrics init failed")?;

    Ok(handle)
}
