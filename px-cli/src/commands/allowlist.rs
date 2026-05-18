use anyhow::{Context, Result, bail};
use px_auth::{AllowlistEntry, AllowlistStore, YamlAllowlistStore};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::cli::AllowlistCmd;

#[derive(Debug, Default, Deserialize, Serialize)]
struct AllowlistFile {
    entries: Vec<AllowlistEntry>,
}

pub async fn run(op: AllowlistCmd) -> Result<()> {
    match op {
        AllowlistCmd::List { path } => list(&path).await,
        AllowlistCmd::Add {
            path,
            domain,
            justification,
            handler,
        } => {
            let entry = AllowlistEntry {
                domain,
                tos_reviewed: true,
                justification,
                handler,
            };
            entry
                .validate()
                .map_err(|e| anyhow::anyhow!("invalid entry: {e}"))?;
            add(&path, entry).await
        }
        AllowlistCmd::Remove { path, domain } => remove(&path, &domain).await,
    }
}

async fn list(path: &Path) -> Result<()> {
    let store = YamlAllowlistStore::load(path)
        .await
        .with_context(|| format!("load {}", path.display()))?;
    for entry in store
        .list()
        .await
        .map_err(|e| anyhow::anyhow!("list: {e}"))?
    {
        let handler = entry.handler.as_deref().unwrap_or("-");
        println!(
            "{}\t{}\t{}\t{}",
            entry.domain, entry.tos_reviewed, handler, entry.justification
        );
    }
    Ok(())
}

async fn add(path: &Path, entry: AllowlistEntry) -> Result<()> {
    let mut file = read_or_empty(path).await?;
    if file.entries.iter().any(|e| e.domain == entry.domain) {
        bail!(
            "domain '{}' already present in {}",
            entry.domain,
            path.display()
        );
    }
    file.entries.push(entry);
    write(path, &file).await
}

async fn remove(path: &Path, domain: &str) -> Result<()> {
    let mut file = read_or_empty(path).await?;
    let before = file.entries.len();
    file.entries.retain(|e| e.domain != domain);
    if file.entries.len() == before {
        bail!("domain '{}' not found in {}", domain, path.display());
    }
    write(path, &file).await
}

async fn read_or_empty(path: &Path) -> Result<AllowlistFile> {
    if !path.exists() {
        return Ok(AllowlistFile::default());
    }
    let bytes = tokio::fs::read(path).await.context("read allowlist")?;
    serde_yaml::from_slice(&bytes).context("parse allowlist")
}

async fn write(path: &Path, file: &AllowlistFile) -> Result<()> {
    let bytes = serde_yaml::to_string(file).context("serialize allowlist")?;
    tokio::fs::write(path, bytes)
        .await
        .context("write allowlist")?;
    Ok(())
}
