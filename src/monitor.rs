// SPDX-License-Identifier: MIT
use serde::Serialize;
use std::{collections::BTreeMap, net::SocketAddr, time::Duration};

const MAX_BODY: usize = 65_536;
const NAMES: [&str; 8] = [
    "xmqr_connections_active",
    "xmqr_sessions",
    "xmqr_retained_messages",
    "xmqr_offline_messages",
    "xmqr_inflight_messages",
    "xmqr_persistence_commits_total",
    "xmqr_persistence_errors_total",
    "xmqr_ready",
];

#[derive(Serialize)]
pub struct Status {
    pub admin_version: &'static str,
    pub reachable: bool,
    pub ready: bool,
    pub error: Option<&'static str>,
    pub metrics: BTreeMap<String, u64>,
}

impl Status {
    fn unavailable(error: &'static str) -> Self {
        Self {
            admin_version: env!("CARGO_PKG_VERSION"),
            reachable: false,
            ready: false,
            error: Some(error),
            metrics: BTreeMap::new(),
        }
    }
}

pub fn client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(1))
        .timeout(Duration::from_secs(2))
        .http1_only()
        .build()
}

pub async fn status(client: &reqwest::Client, address: SocketAddr) -> Status {
    let result = tokio::time::timeout(Duration::from_secs(3), observe(client, address)).await;
    match result {
        Ok(Ok(status)) => status,
        Ok(Err(error)) => Status::unavailable(error),
        Err(_) => Status::unavailable("monitor_timeout"),
    }
}

async fn observe(client: &reqwest::Client, address: SocketAddr) -> Result<Status, &'static str> {
    let ready = client
        .get(format!("http://{address}/ready"))
        .send()
        .await
        .map_err(|_| "monitor_unavailable")?;
    let code = ready.status().as_u16();
    if !matches!(code, 200 | 503) {
        return Err("monitor_invalid_response");
    }
    drop(ready);
    let mut response = client
        .get(format!("http://{address}/metrics"))
        .send()
        .await
        .map_err(|_| "monitor_unavailable")?;
    if response.status().as_u16() != 200
        || response
            .content_length()
            .is_some_and(|n| n > MAX_BODY as u64)
    {
        return Err("monitor_invalid_response");
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "monitor_invalid_response")?
    {
        if chunk.len() > MAX_BODY - bytes.len() {
            return Err("monitor_response_too_large");
        }
        bytes.extend_from_slice(&chunk);
    }
    let metrics = parse(std::str::from_utf8(&bytes).map_err(|_| "monitor_invalid_metrics")?)?;
    Ok(Status {
        admin_version: env!("CARGO_PKG_VERSION"),
        reachable: true,
        ready: code == 200 && metrics["xmqr_ready"] == 1,
        error: None,
        metrics,
    })
}

fn parse(text: &str) -> Result<BTreeMap<String, u64>, &'static str> {
    let mut metrics = BTreeMap::new();
    for line in text.lines() {
        let mut parts = line.split_ascii_whitespace();
        let Some(name) = parts.next() else { continue };
        if !NAMES.contains(&name) {
            continue;
        }
        let value = parts
            .next()
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or("monitor_invalid_metrics")?;
        if parts.next().is_some() || metrics.insert(name.to_owned(), value).is_some() {
            return Err("monitor_invalid_metrics");
        }
    }
    if metrics.len() != NAMES.len() || metrics["xmqr_ready"] > 1 {
        return Err("monitor_invalid_metrics");
    }
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> String {
        use std::fmt::Write;
        NAMES.iter().fold(String::new(), |mut text, name| {
            writeln!(text, "{name} 1").unwrap();
            text
        })
    }

    #[test]
    fn only_fixed_numeric_aggregates_are_returned() {
        let text = sample() + "unknown_secret{user=\"not-returned\"} 9\n";
        let metrics = parse(&text).unwrap();
        assert_eq!(metrics.len(), 8);
        assert!(!metrics.contains_key("unknown_secret"));
    }

    #[test]
    fn missing_duplicate_negative_noninteger_or_invalid_readiness_is_not_zero() {
        for text in [
            String::new(),
            sample() + "xmqr_sessions 0\n",
            sample().replace("xmqr_sessions 1", "xmqr_sessions -1"),
            sample().replace("xmqr_sessions 1", "xmqr_sessions NaN"),
            sample().replace("xmqr_ready 1", "xmqr_ready 2"),
        ] {
            assert!(parse(&text).is_err());
        }
    }
}
