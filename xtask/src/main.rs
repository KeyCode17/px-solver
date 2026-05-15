use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "xtask", about = "px-solver dev automation")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    Bump {
        #[arg(value_enum)]
        kind: BumpKind,
        phase: Option<String>,
        #[arg(long)]
        skip_gate: bool,
    },
    CheckLoc {
        #[arg(default_value_t = 200)]
        max: usize,
    },
    Release {
        #[arg(long)]
        remote: Option<String>,
    },
    Canary,
    Phase {
        name: String,
        #[arg(long)]
        skip_gate: bool,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum BumpKind {
    Major,
    Minor,
    Patch,
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Bump {
            kind,
            phase,
            skip_gate,
        } => bump(kind, phase, skip_gate),
        Cmd::CheckLoc { max } => check_loc(max),
        Cmd::Release { remote } => release(remote),
        Cmd::Canary => canary(),
        Cmd::Phase { name, skip_gate } => phase(name, skip_gate),
    }
}

fn phase(name: String, skip_gate: bool) -> Result<()> {
    let kind = match name.as_str() {
        "00" | "01" | "02" | "03" => BumpKind::Minor,
        "04" => BumpKind::Major,
        n if n.starts_with('R') => {
            println!("[xtask] Phase {n} is research-track; no version bump (ADR-0013/0017)");
            return Ok(());
        }
        n => bail!("unknown phase '{n}'; expected 00|01|02|03|04|R*"),
    };
    bump(kind, Some(name), skip_gate)
}

fn bump(kind: BumpKind, phase: Option<String>, skip_gate: bool) -> Result<()> {
    let manifest_path = Path::new("Cargo.toml");
    let manifest = fs::read_to_string(manifest_path).context("read root Cargo.toml")?;
    let cur = parse_workspace_version(&manifest)?;
    let next = cur.bump(kind);

    let from = format!("version       = \"{cur}\"");
    let to = format!("version       = \"{next}\"");
    let new = manifest.replacen(&from, &to, 1);
    if new == manifest {
        bail!("could not find `{from}` in Cargo.toml");
    }
    fs::write(manifest_path, new).context("write root Cargo.toml")?;

    if !skip_gate {
        run("cargo", &["build", "--workspace", "--all-targets"])?;
        run("cargo", &["test", "--workspace", "--no-fail-fast"])?;
    }

    let msg = match phase {
        Some(p) => format!("chore: bump to {next} (Phase {p})"),
        None => format!("chore: bump to {next}"),
    };
    run("git", &["commit", "-am", &msg])?;
    run("git", &["tag", "-a", &format!("v{next}"), "-m", &msg])?;
    println!("[xtask] bumped to {next}");
    Ok(())
}

fn check_loc(max: usize) -> Result<()> {
    let mut fail = 0usize;
    for entry in walkdir::WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if p.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let s = p.to_string_lossy();
        if s.contains("/target/")
            || s.contains("/tests/")
            || s.contains("/examples/")
            || s.contains("/xtask/")
            || s.contains("/px-research/")
        {
            continue;
        }
        let lines = fs::read_to_string(p)
            .map(|s| s.lines().count())
            .unwrap_or(0);
        if lines > max {
            eprintln!("X {} : {lines} LOC (max {max})", p.display());
            fail += 1;
        }
    }
    if fail > 0 {
        bail!("{fail} file(s) over LOC limit");
    }
    Ok(())
}

fn release(remote: Option<String>) -> Result<()> {
    let manifest = fs::read_to_string("Cargo.toml").context("read root Cargo.toml")?;
    let cur = parse_workspace_version(&manifest)?;
    let tag = format!("v{cur}");
    let r = remote.as_deref().unwrap_or("origin");
    run("git", &["push", r, "HEAD"])?;
    run("git", &["push", r, &tag])?;
    println!("[xtask] released {tag} to {r}");
    Ok(())
}

fn canary() -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "test",
            "-p",
            "px-server",
            "--test",
            "pedidosya",
            "--",
            "--nocapture",
        ])
        .env("CI_CANARY", "1")
        .status()
        .context("spawn cargo test canary")?;
    if !status.success() {
        bail!("canary test failed: {status}");
    }
    println!("[xtask] canary passed");
    Ok(())
}

fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("spawn {cmd}"))?;
    if !status.success() {
        bail!("{cmd} {args:?} failed: {status}");
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct Version {
    x: u32,
    y: u32,
    z: u32,
}

impl Version {
    fn bump(self, kind: BumpKind) -> Self {
        match kind {
            BumpKind::Major => Self {
                x: self.x + 1,
                y: 0,
                z: 0,
            },
            BumpKind::Minor => Self {
                x: self.x,
                y: self.y + 1,
                z: 0,
            },
            BumpKind::Patch => Self {
                x: self.x,
                y: self.y,
                z: self.z + 1,
            },
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.x, self.y, self.z)
    }
}

fn parse_workspace_version(manifest: &str) -> Result<Version> {
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("version") && l.contains('"'))
        .ok_or_else(|| anyhow!("no version line in [workspace.package]"))?;
    let between = line
        .split('"')
        .nth(1)
        .ok_or_else(|| anyhow!("malformed version line"))?;
    let mut parts = between.split('.');
    let x: u32 = parts
        .next()
        .ok_or_else(|| anyhow!("missing major"))?
        .parse()?;
    let y: u32 = parts
        .next()
        .ok_or_else(|| anyhow!("missing minor"))?
        .parse()?;
    let z: u32 = parts
        .next()
        .ok_or_else(|| anyhow!("missing patch"))?
        .parse()?;
    Ok(Version { x, y, z })
}
