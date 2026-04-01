//! LSP Daemon - Persistent background LSP process
//!
//! The daemon solves the "cold start" problem: each CLI invocation would otherwise
//! spawn a new LSP server (2-3s startup cost). Instead, the daemon:
//!
//! 1. Starts once on first query
//! 2. Keeps LSP process alive between commands
//! 3. Shares cache across invocations
//! 4. Auto-restarts on crash
//!
//! ## Architecture
//!
//! ```text
//! CLI Command 1 → DaemonClient → (start daemon) → persistent LSP → query
//! CLI Command 2 → DaemonClient → (reuse daemon) → check cache → return
//! CLI Command 3 → DaemonClient → (reuse daemon) → LSP query (cache miss)
//! ```

pub mod process;

pub use process::LspDaemon;
