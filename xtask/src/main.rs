use std::{
    path::Path,
    process::{Command, ExitStatus},
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask")]
struct Args {
    #[command(subcommand)]
    command: XtaskCommand,
}

#[derive(Subcommand)]
enum XtaskCommand {
    /// Run library unit tests under Miri.
    Miri,
}

fn main() -> Result<()> {
    match Args::parse().command {
        XtaskCommand::Miri => miri(),
    }
}

fn miri() -> Result<()> {
    ensure_success(
        cargo_nightly()
            .args(["miri", "test", "--lib", "--locked"])
            .status()
            .context("failed to spawn `cargo miri`")?,
    )
}

fn cargo_nightly() -> Command {
    let mut cmd = Command::new("rustup");
    cmd.args(["run", "nightly", "cargo"]);
    cmd.current_dir(project_root());
    cmd
}

fn project_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

fn ensure_success(status: ExitStatus) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        bail!("process exited with {status}");
    }
}
