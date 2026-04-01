//! # TypeGlass CLI
//!
//! TypeGlass is a command-line tool for exploring type relationships in codebases using
//! Language Server Protocol (LSP). It builds type dependency graphs on-demand by querying
//! LSP servers, enabling lazy navigation of type hierarchies without analyzing entire
//! projects upfront.
//!
//! ## Architecture
//!
//! TypeGlass follows **hexagonal (ports & adapters) architecture**:
//!
//! - **Domain Layer** (`domain`) - Pure business logic with no infrastructure dependencies
//!   - Type graph models (TypeNode, TypeEdge, TypeGraph)
//!   - Traversal engine (BFS with cycle detection)
//!   - Smart constructors preventing invalid states
//!   - All types use Result<T, E> for error handling
//!
//! - **Application Layer** (`application`) - Use case orchestration
//!   - Commands: `from`, `gc`, `init`, `interactive`, `doctor`
//!   - Service layer coordinating ports
//!   - UX model for user intent and results
//!
//! - **Infrastructure Layer** (`infrastructure`) - External adapters
//!   - LSP client (JSON-RPC 2.0 protocol)
//!   - Graph cache (file-based with TTL)
//!   - CLI argument parsing
//!   - Filesystem scanners
//!
//! - **Ports** (`domain::ports`) - Abstract interfaces
//!   - LspPort, WorkspacePort, ClockPort, etc.
//!   - Implemented by infrastructure adapters
//!
//! ## Key Features
//!
//! - **Lazy Graph Building** - Only queries types as needed via LSP
//! - **Multi-Language** - Works with Rust, TypeScript, Go (any LSP-enabled language)
//! - **Smart Caching** - 5-minute TTL cache for repeated queries
//! - **Type Safety** - Smart constructors, branded types, exhaustive matching
//! - **Interactive Mode** - REPL for exploration
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use typeglass_cli::domain::graph::{TypeGraph, SymbolName, TraversalDirection};
//! use typeglass_cli::infrastructure::LazyGraphBuilder;
//! use typeglass_cli::domain::language::Language;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize LSP client for a Rust project
//!     let workspace_root = PathBuf::from("/path/to/project");
//!     let mut builder = LazyGraphBuilder::new(workspace_root, Language::Rust).await?;
//!     
//!     // Build type graph from a symbol
//!     let graph = builder.build_from_symbol("MyType", TraversalDirection::Both, 5).await?;
//!     
//!     println!("Found {} types", graph.nodes().len());
//!     
//!     builder.shutdown().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Design Principles
//!
//! 1. **Type-Level Safety** - Invalid states are unrepresentable
//! 2. **Explicit Errors** - All errors use Result, no panics
//! 3. **Immutability Default** - Mutation only where necessary
//! 4. **No Dynamic Dispatch** - Monomorfización over `dyn Trait`
//! 5. **Hexagonal Architecture** - Domain never imports infrastructure

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ux_model;
