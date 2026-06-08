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
    /// Run the test suite with cargo-nextest.
    Test {
        /// Extra arguments forwarded to `cargo nextest run`.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() -> Result<()> {
    match Args::parse().command {
        XtaskCommand::Miri => miri(),
        XtaskCommand::Test { args } => test(&args),
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

fn test(args: &[String]) -> Result<()> {
    let mut cmd = cargo();
    cmd.args(["nextest", "run", "--locked"]);
    cmd.args(args);
    ensure_success(
        cmd.status()
            .context("failed to spawn `cargo nextest run`")?,
    )
}

fn cargo() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(project_root());
    cmd
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
