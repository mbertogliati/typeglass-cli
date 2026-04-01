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
#[command(after_help = "TIP: Use --help after any command for detailed options (e.g., 'typeglass from --help')")]
pub struct CliArgs {
    /// Output results as JSON (default: human-readable)
    #[arg(long, global = true, help = "Output as JSON instead of human-readable format")]
    pub json: bool,
    
    /// Enable debug logging (shows LSP communication details)
    #[arg(long, global = true, help = "Enable verbose debug output")]
    pub debug: bool,
    
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
    /// Name of a type, struct, class, interface, enum, or function
    #[arg(
        long, 
        value_name = "NAME",
        help = "Symbol name (e.g., TypeNode, HashMap, UserService)",
        long_help = "Name of a type, struct, class, interface, enum, or function to start navigation from.

Examples by language:
  Rust:       TypeNode, Result, Option, HashMap, Vec
  TypeScript: UserService, ApiClient, React.Component
  Go:         http.Server, context.Context, sql.DB

How to find symbols:
  • Check your editor's 'Go to Symbol' (Cmd+T / Ctrl+T)
  • Look at type definitions in your code
  • Use grep: grep -r '^struct \\|^class \\|^type ' src/
  • Just try a type name - the CLI will tell you if not found"
    )]
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
    
    /// Output format
    #[arg(
        long,
        value_enum,
        default_value = "human",
        help = "Output format (human, json, dot, mermaid)"
    )]
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    Human,
    /// JSON format
    Json,
    /// Graphviz DOT format
    Dot,
    /// Mermaid diagram format
    Mermaid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFromCommand {
    pub target: QueryTarget,
    pub depth: Option<u8>,
    pub format: OutputFormat,
}

#[derive(Debug, Error)]
pub enum FromArgsError {
    #[error("A query target is required: use one of --symbol, --file, --module, --public-exports")]
    MissingTarget,
    #[error("Only one query target is allowed at a time")]
    MultipleTargets,
}
