use anyhow::{Context, Result, bail};
use px_detector::{Detected, Detector, RegexDetector};
use tokio::io::AsyncReadExt;

use crate::cli::DetectArgs;

pub async fn run(args: DetectArgs) -> Result<()> {
    let html = match args.url {
        Some(url) => fetch_html(&url).await?,
        None => read_stdin_html().await?,
    };
    match RegexDetector::new().detect(&html) {
        Detected::Yes(d) => {
            let serialized = serde_yaml::to_string(&d).context("serialize detection")?;
            println!("{serialized}");
            Ok(())
        }
        Detected::No => bail!("no PerimeterX markers detected on input"),
    }
}

async fn read_stdin_html() -> Result<String> {
    let mut buf = Vec::new();
    tokio::io::stdin()
        .read_to_end(&mut buf)
        .await
        .context("read stdin")?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

async fn fetch_html(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .gzip(true)
        .user_agent(default_user_agent())
        .build()
        .context("build http client")?;
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;
    let status = resp.status();
    let body = resp.text().await.context("read response body")?;
    // PX block pages return 4xx with the markers embedded in the body. Keep
    // the body so the detector can still surface `Detected::Yes`; bail only
    // on hard transport errors (5xx, redirects we already followed, etc.).
    if status.is_server_error() {
        bail!("upstream returned HTTP {status} for {url}");
    }
    Ok(body)
}

fn default_user_agent() -> &'static str {
    concat!(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 ",
        "(KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36"
    )
}
