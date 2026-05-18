mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Cmd};

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Detect(args) => commands::detect::run(args).await,
        Cmd::Keys { op } => commands::keys::run(op).await,
        Cmd::Allowlist { op } => commands::allowlist::run(op).await,
        Cmd::Serve => commands::serve::run(),
        Cmd::Solve(args) => commands::solve::run(args).await,
    }
}
