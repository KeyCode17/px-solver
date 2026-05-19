use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "px-cli", about = "Operator CLI for px-solver")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Run the PerimeterX detector against a URL or stdin HTML.
    Detect(DetectArgs),
    /// Manage API keys.
    Keys {
        #[command(subcommand)]
        op: KeysCmd,
    },
    /// Manage the per-domain allowlist.
    Allowlist {
        #[command(subcommand)]
        op: AllowlistCmd,
    },
    /// Convenience pointer to `cargo run -p px-server`.
    Serve,
    /// Solve a target URL by calling a running px-server's POST /v1/solve.
    Solve(SolveArgs),
    /// Diff a Camoufox sensor capture against px-native's synthetic batch.
    Calibrate(CalibrateArgs),
}

#[derive(Args, Debug)]
pub struct DetectArgs {
    /// URL to fetch and inspect. If absent, HTML is read from stdin.
    #[arg(long)]
    pub url: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum KeysCmd {
    /// Generate a new key id + secret + argon2 hash.
    Generate {
        #[arg(long)]
        id: String,
        #[arg(long)]
        note: Option<String>,
        /// If set, append the generated hash to the keys file.
        #[arg(long, default_value = "config/keys.yaml")]
        path: PathBuf,
        /// Append to the keys file instead of just printing.
        #[arg(long)]
        write: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum AllowlistCmd {
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
        /// Optional handler routing hint, e.g. `cloudflare` (ADR-0023).
        #[arg(long)]
        handler: Option<String>,
    },
    Remove {
        #[arg(long, default_value = "config/allowlist.yaml")]
        path: PathBuf,
        #[arg(long)]
        domain: String,
    },
}

#[derive(Args, Debug)]
pub struct SolveArgs {
    /// Target URL to solve.
    pub url: String,
    /// Base URL of a running px-server.
    #[arg(long, env = "PX_SERVER_URL", default_value = "http://127.0.0.1:8080")]
    pub server: String,
    /// API key as `id:secret`.
    #[arg(long, env = "PX_API_KEY")]
    pub api_key: String,
    /// Optional upstream proxy passed to the solver.
    #[arg(long)]
    pub proxy: Option<String>,
}

#[derive(Args, Debug)]
pub struct CalibrateArgs {
    /// Path to a capture JSON emitted by `pxsolver-camoufox::capture_sensor`.
    pub capture: PathBuf,
    /// Print the report as JSON (default: human-readable).
    #[arg(long)]
    pub json: bool,
}
