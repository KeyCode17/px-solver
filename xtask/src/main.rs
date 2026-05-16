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
        #[arg(long)]
        local: bool,
        #[arg(long)]
        remote: Option<String>,
        #[arg(long)]
        title: Option<String>,
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
    Soak {
        #[arg(long, default_value = "24h")]
        duration: String,
        #[arg(long, default_value_t = 1)]
        rps: u32,
        #[arg(long, default_value = "https://www.havenwellwithin.com/")]
        target: String,
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
    },
    Phase {
        name: String,
        #[arg(long)]
        skip_gate: bool,
        #[arg(long)]
        local: bool,
        #[arg(long)]
        remote: Option<String>,
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
            local,
            remote,
            title,
        } => bump(kind, phase, skip_gate, local, remote, title),
        Cmd::CheckLoc { max } => check_loc(max),
        Cmd::Release { remote } => release(remote),
        Cmd::Canary => canary(),
        Cmd::Soak {
            duration,
            rps,
            target,
            server,
        } => soak(duration, rps, target, server),
        Cmd::Phase {
            name,
            skip_gate,
            local,
            remote,
        } => phase(name, skip_gate, local, remote),
    }
}

fn phase(name: String, skip_gate: bool, local: bool, remote: Option<String>) -> Result<()> {
    let kind = match name.as_str() {
        "00" | "01" | "02" | "03" => BumpKind::Minor,
        "04" => BumpKind::Major,
        n if n.starts_with('R') => {
            println!("[xtask] Phase {n} is research-track; no version bump (ADR-0013/0017)");
            return Ok(());
        }
        n => bail!("unknown phase '{n}'; expected 00|01|02|03|04|R*"),
    };
    bump(kind, Some(name), skip_gate, local, remote, None)
}

fn bump(
    kind: BumpKind,
    phase: Option<String>,
    skip_gate: bool,
    local: bool,
    remote: Option<String>,
    title: Option<String>,
) -> Result<()> {
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

    let msg = match phase.as_deref() {
        Some(p) => format!("chore: bump to {next} (Phase {p})"),
        None => format!("chore: bump to {next}"),
    };
    let tag = format!("v{next}");
    run("git", &["commit", "-am", &msg])?;
    run("git", &["tag", "-a", &tag, "-m", &msg])?;
    println!("[xtask] bumped to {next}");

    if local {
        println!("[xtask] --local set; skipping push + GitHub release");
        return Ok(());
    }
    let body = release_body(&next, kind, phase.as_deref(), title.as_deref());
    publish(&tag, &body, remote.as_deref().unwrap_or("origin"))
}

fn release_body(
    version: &Version,
    kind: BumpKind,
    phase: Option<&str>,
    title: Option<&str>,
) -> String {
    let header = if let Some(p) = phase {
        format!("# {}", phase_title(p))
    } else if let Some(t) = title {
        format!("# {t}")
    } else {
        format!("# v{version}")
    };
    let body = if let Some(p) = phase {
        phase_description(p)
    } else {
        match kind {
            BumpKind::Major => "Major release.".into(),
            BumpKind::Minor => "Minor release.".into(),
            BumpKind::Patch => "Maintenance bump.".into(),
        }
    };
    format!("{header}\n\n{body}\n")
}

fn phase_title(name: &str) -> String {
    match name {
        "00" => "Phase 00 \u{2014} Bootstrap".into(),
        "01" => "Phase 01 \u{2014} Foundation".into(),
        "02" => "Phase 02 \u{2014} Harvester + PX handler".into(),
        "03" => "Phase 03 \u{2014} Server + Auth + Native stub".into(),
        "04" => "Phase 04 \u{2014} CLI + Canary + Docs (MVP)".into(),
        n => format!("Phase {n}"),
    }
}

fn phase_description(name: &str) -> String {
    match name {
        "00" => "Workspace scaffold, rust-toolchain 1.95, lefthook, CI, xtask. SOW-DEL-001.".into(),
        "01" => "Domain types, PX detector, cookie cache, challenge pipeline, handler stubs. SOW-DEL-002/004/013-017.".into(),
        "02" => "Chromium pool + 8-patch stealth bundle + PerimeterX handler + _pxhd parser. SOW-DEL-003a/003b. Phase 02 exit gates (>=95% solve, p50 <=6s, zero zombies, FP regressions) are not verified in this build; awaits operator soak.".into(),
        "03" => "Axum REST API, API-key auth, allowlist, audit log, native stub. SOW-DEL-005/006/009.".into(),
        "04" => "Operator CLI, canary integration test, operator docs. MVP-AC-1..7 hold. SOW-DEL-007/008/010.".into(),
        n => format!("Phase {n} release."),
    }
}

fn publish(tag: &str, body: &str, remote: &str) -> Result<()> {
    run("git", &["push", remote, "HEAD"])?;
    run("git", &["push", remote, tag])?;
    let body_file = std::env::temp_dir().join(format!("xtask-release-{tag}.md"));
    fs::write(&body_file, body).context("write release body file")?;
    let body_path = body_file.to_string_lossy();
    let status = Command::new("gh")
        .args([
            "release",
            "create",
            tag,
            "--title",
            tag,
            "--notes-file",
            body_path.as_ref(),
        ])
        .status()
        .context("spawn gh release create")?;
    let _ = fs::remove_file(&body_file);
    if !status.success() {
        bail!("gh release create failed: {status}");
    }
    println!("[xtask] published {tag} to {remote}");
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
    let body = format!("# {tag}\n\nMaintenance bump.\n");
    publish(&tag, &body, remote.as_deref().unwrap_or("origin"))
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

fn soak(duration: String, rps: u32, target: String, server: String) -> Result<()> {
    if std::env::var("PX_SOAK_KEY").is_err() {
        bail!("PX_SOAK_KEY must be set to '<id>:<secret>' (matching a record in config/keys.yaml)");
    }
    let script = Path::new("scripts/soak.sh");
    if !script.exists() {
        bail!("scripts/soak.sh not found; run from workspace root");
    }
    let status = Command::new("bash")
        .arg(script)
        .arg(&duration)
        .arg(rps.to_string())
        .arg(&target)
        .env("PX_SERVER", &server)
        .status()
        .context("spawn scripts/soak.sh")?;
    match status.code() {
        Some(0) => {
            println!("[xtask] soak finished: all MVP-AC-1..4 passed");
            Ok(())
        }
        Some(1) => bail!("soak finished: one or more AC-1..4 failed (see evidence file)"),
        Some(c) => bail!("soak harness errored (exit {c})"),
        None => bail!("soak harness terminated by signal"),
    }
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
