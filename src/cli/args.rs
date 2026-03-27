use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryTarget {
    Symbol(String),
    File(PathBuf),
    Module(PathBuf),
    PublicExports,
}

#[derive(Debug, Parser)]
#[command(name = "typeglass")]
#[command(about = "Navigate type relationships lazily through LSP")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Debug, Subcommand)]
pub enum CliCommand {
    From(FromArgs),
    Daemon(DaemonArgs),
    Gc,
    Doctor,
    Init,
    Interactive,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("target")
        .args(["symbol", "file", "module", "public_exports"])
        .required(true)
        .multiple(false)
))]
pub struct FromArgs {
    #[arg(long)]
    pub symbol: Option<String>,
    #[arg(long)]
    pub file: Option<PathBuf>,
    #[arg(long)]
    pub module: Option<PathBuf>,
    #[arg(long)]
    pub public_exports: bool,
    #[arg(long)]
    pub depth: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFromCommand {
    pub target: QueryTarget,
    pub depth: Option<u8>,
}

#[derive(Debug, Error)]
pub enum FromArgsError {
    #[error("A query target is required: use one of --symbol, --file, --module, --public-exports")]
    MissingTarget,
    #[error("Only one query target is allowed at a time")]
    MultipleTargets,
}

#[derive(Debug, Args)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub action: DaemonCommand,
}

#[derive(Debug, Subcommand)]
pub enum DaemonCommand {
    Start,
    Stop,
    Status,
}
