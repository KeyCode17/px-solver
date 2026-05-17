use anyhow::{Context, Result, bail};
use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString};
use clap::{Parser, Subcommand};
use px_auth::{AllowlistEntry, AllowlistStore, YamlAllowlistStore};
use px_detector::{Detected, Detector, RegexDetector};
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

#[derive(Parser, Debug)]
#[command(name = "px-cli", about = "Operator CLI for px-solver")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Detect,
    Keys {
        #[command(subcommand)]
        op: KeysCmd,
    },
    Allowlist {
        #[command(subcommand)]
        op: AllowlistCmd,
    },
    Serve,
    Solve {
        url: String,
    },
}

#[derive(Subcommand, Debug)]
enum KeysCmd {
    Generate {
        #[arg(long)]
        id: String,
        #[arg(long)]
        note: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum AllowlistCmd {
    List {
        #[arg(long, default_value = "config/allowlist.yaml")]
        path: PathBuf,
    },
    Add {
        #[arg(long, default_value = "config/allowlist.yaml")]
        path: PathBuf,
        #[arg(long)]
        domain: String,
        #[arg(long)]
        justification: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Detect => detect_from_stdin().await,
        Cmd::Keys { op } => keys(op).await,
        Cmd::Allowlist { op } => allowlist(op).await,
        Cmd::Serve => serve_stub(),
        Cmd::Solve { url } => solve_stub(&url),
    }
}

async fn detect_from_stdin() -> Result<()> {
    let mut buf = Vec::new();
    tokio::io::stdin()
        .read_to_end(&mut buf)
        .await
        .context("read stdin")?;
    let html = String::from_utf8_lossy(&buf);
    let detector = RegexDetector::new();
    match detector.detect(&html) {
        Detected::Yes(d) => {
            println!(
                "{}",
                serde_yaml::to_string(&d).context("serialize detection")?
            );
            Ok(())
        }
        Detected::No => bail!("no PerimeterX markers detected on input"),
    }
}

async fn keys(op: KeysCmd) -> Result<()> {
    let KeysCmd::Generate { id, note } = op;
    let secret = uuid::Uuid::new_v4().simple().to_string();
    let salt =
        SaltString::encode_b64(secret.as_bytes()).map_err(|e| anyhow::anyhow!("salt: {e}"))?;
    let hash = Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash: {e}"))?
        .to_string();
    println!("id: {id}");
    println!("secret: {secret}");
    println!("argon2_hash: {hash}");
    if let Some(n) = note {
        println!("note: {n}");
    }
    Ok(())
}

async fn allowlist(op: AllowlistCmd) -> Result<()> {
    match op {
        AllowlistCmd::List { path } => {
            let store = YamlAllowlistStore::load(&path)
                .await
                .with_context(|| format!("load {}", path.display()))?;
            for entry in store
                .list()
                .await
                .map_err(|e| anyhow::anyhow!("list: {e}"))?
            {
                println!(
                    "{}\t{}\t{}",
                    entry.domain, entry.tos_reviewed, entry.justification
                );
            }
            Ok(())
        }
        AllowlistCmd::Add {
            path,
            domain,
            justification,
        } => {
            let entry = AllowlistEntry {
                domain,
                tos_reviewed: true,
                justification,
                handler: None,
            };
            append_allowlist(&path, entry).await
        }
    }
}

async fn append_allowlist(path: &std::path::Path, entry: AllowlistEntry) -> Result<()> {
    #[derive(Debug, serde::Deserialize, serde::Serialize, Default)]
    struct File {
        entries: Vec<AllowlistEntry>,
    }
    let mut file: File = if path.exists() {
        let bytes = tokio::fs::read(path).await.context("read allowlist")?;
        serde_yaml::from_slice(&bytes).context("parse allowlist")?
    } else {
        File::default()
    };
    file.entries.push(entry);
    let bytes = serde_yaml::to_string(&file).context("serialize allowlist")?;
    tokio::fs::write(path, bytes)
        .await
        .context("write allowlist")?;
    Ok(())
}

fn serve_stub() -> Result<()> {
    println!(
        "px-cli serve is a thin wrapper; run the px-server binary directly: `cargo run -p px-server`"
    );
    Ok(())
}

fn solve_stub(url: &str) -> Result<()> {
    println!(
        "px-cli solve is not bound to a running server in this build; POST to /v1/solve at the server with {{\"url\":\"{url}\"}}"
    );
    Ok(())
}
