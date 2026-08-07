use anyhow::{Context, Result, bail};
use px_core::SolveRequest;
use serde::Deserialize;

use crate::cli::SolveArgs;

#[derive(Debug, Deserialize)]
struct SolveEnvelope {
    data: serde_json::Value,
    #[serde(default)]
    status: Option<String>,
}

pub async fn run(args: SolveArgs) -> Result<()> {
    let SolveArgs {
        url,
        server,
        api_key,
        proxy,
    } = args;
    if !api_key.contains(':') {
        bail!("--api-key (or PX_API_KEY) must be in the form `id:secret`");
    }
    let endpoint = format!("{}/v1/solve", server.trim_end_matches('/'));
    let body = SolveRequest::new(&url).with_proxy_opt(proxy);
    let client = reqwest::Client::builder()
        .build()
        .context("build http client")?;
    let resp = client
        .post(&endpoint)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("POST {endpoint}"))?;
    let status = resp.status();
    let text = resp.text().await.context("read response body")?;
    if !status.is_success() {
        bail!("px-server returned HTTP {status}: {text}");
    }
    let envelope: SolveEnvelope =
        serde_json::from_str(&text).context("parse SingleResponse<SolveResponseDto>")?;
    let pretty = serde_json::to_string_pretty(&envelope.data).context("serialize data")?;
    println!("{pretty}");
    if let Some(s) = envelope.status.as_deref() {
        eprintln!("status: {s}");
    }
    Ok(())
}
