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

#[derive(Debug, Parser, Clone)]
#[command(name = "typeglass")]
#[command(about = "Navigate type relationships lazily through LSP")]
#[command(version)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum CliCommand {
    /// Navigate type relationships from a starting point
    #[command(about = "Build a lazy type graph starting from a symbol, file, or module")]
    From(FromArgs),
    
    /// Clean up cached type graphs
    #[command(about = "Remove expired or invalid cached graphs")]
    Gc,
    
    /// Check LSP server availability
    #[command(about = "Verify that required LSP servers are installed and accessible")]
    Doctor,
    
    /// Initialize a new workspace
    #[command(about = "Set up typeglass configuration for a workspace")]
    Init,
    
    /// Start interactive exploration mode
    #[command(about = "Launch an interactive shell for type navigation")]
    Interactive,
}

#[derive(Debug, Args, Clone)]
#[command(group(
    ArgGroup::new("target")
        .args(["symbol", "file", "module", "public_exports"])
        .required(true)
        .multiple(false)
))]
pub struct FromArgs {
    /// Start from a specific symbol (type, function, etc.)
    #[arg(long, help = "Symbol name to start navigation from (e.g., TypeNode)")]
    pub symbol: Option<String>,
    
    /// Start from all symbols in a file
    #[arg(long, help = "File path to analyze")]
    pub file: Option<PathBuf>,
    
    /// Start from all symbols in a module
    #[arg(long, help = "Module path to analyze")]
    pub module: Option<PathBuf>,
    
    /// Start from all public exports
    #[arg(long, help = "Analyze all publicly exported symbols")]
    pub public_exports: bool,
    
    /// Maximum depth to traverse
    #[arg(long, help = "How many levels deep to traverse type relationships (default: 1)", default_value = "1")]
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
