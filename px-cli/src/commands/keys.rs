use anyhow::{Context, Result};
use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::cli::KeysCmd;

#[derive(Debug, Default, Deserialize, Serialize)]
struct KeysFile {
    #[serde(default)]
    keys: Vec<KeyEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct KeyEntry {
    id: String,
    argon2_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

pub async fn run(op: KeysCmd) -> Result<()> {
    let KeysCmd::Generate {
        id,
        note,
        path,
        write,
    } = op;
    let (secret, hash) = generate_secret_and_hash()?;
    println!("id: {id}");
    println!("secret: {secret}");
    println!("argon2_hash: {hash}");
    if let Some(n) = &note {
        println!("note: {n}");
    }
    if write {
        append_key(
            &path,
            KeyEntry {
                id,
                argon2_hash: hash,
                note,
            },
        )
        .await?;
        println!("written: {}", path.display());
    } else {
        println!(
            "(not persisted; re-run with --write to append to {})",
            path.display()
        );
    }
    Ok(())
}

fn generate_secret_and_hash() -> Result<(String, String)> {
    let secret = uuid::Uuid::new_v4().simple().to_string();
    let salt =
        SaltString::encode_b64(secret.as_bytes()).map_err(|e| anyhow::anyhow!("salt: {e}"))?;
    let hash = Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash: {e}"))?
        .to_string();
    Ok((secret, hash))
}

async fn append_key(path: &Path, entry: KeyEntry) -> Result<()> {
    let mut file: KeysFile = if path.exists() {
        let bytes = tokio::fs::read(path).await.context("read keys file")?;
        serde_yaml::from_slice(&bytes).context("parse keys file")?
    } else {
        KeysFile::default()
    };
    if file.keys.iter().any(|k| k.id == entry.id) {
        anyhow::bail!(
            "key id '{}' already present in {}",
            entry.id,
            path.display()
        );
    }
    file.keys.push(entry);
    let bytes = serde_yaml::to_string(&file).context("serialize keys file")?;
    tokio::fs::write(path, bytes)
        .await
        .context("write keys file")?;
    Ok(())
}
