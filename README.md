# typeglass

[![CI](https://github.com/YOUR_USERNAME/typeglass-cli/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/typeglass-cli/actions)
[![codecov](https://codecov.io/gh/YOUR_USERNAME/typeglass-cli/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/typeglass-cli)
[![Crates.io](https://img.shields.io/crates/v/typeglass-cli.svg)](https://crates.io/crates/typeglass-cli)
[![License](https://img.shields.io/crates/l/typeglass-cli.svg)](LICENSE)

**Navigate type relationships lazily through LSP**

`typeglass` is a CLI tool that builds type dependency graphs on-demand using Language Server Protocol (LSP). Instead of analyzing entire codebases upfront, it queries LSP servers lazily to explore type relationships as you navigate.

## Features

- 🔍 **Lazy graph building** - Query type dependencies on-demand via LSP
- 🌐 **Multi-language support** - Rust, TypeScript, Go, Java, Kotlin (any language with LSP server)
- 🧬 **Semantic relationships** - Accurate EdgeKind detection (Extends, Implements, Instantiates, Contains) via hover analysis
- 📊 **Multiple output formats** - Human-readable, JSON, Graphviz DOT, and Mermaid diagrams
- 🔄 **Recursive traversal** - BFS traversal with configurable depth
- 🎯 **Cycle detection** - Prevents infinite loops with visited tracking
- 💾 **Smart caching** - File-based cache with 5-minute TTL
- 🏥 **Health checks** - `doctor` command verifies LSP availability
- 💬 **Helpful errors** - Clear error messages with actionable next steps

## Installation

### From crates.io

```bash
cargo install typeglass-cli
```

### From source

```bash
git clone https://github.com/YOUR_USERNAME/typeglass-cli
cd typeglass-cli
cargo build --release
# Binary available at target/release/typeglass
```

### Prerequisites

Install a language server for your target language:

- **Rust**: `rustup component add rust-analyzer`
- **TypeScript**: `npm install -g typescript-language-server typescript`
- **Go**: `go install golang.org/x/tools/gopls@latest`
- **Java**: `brew install jdtls` or download from [eclipse.org/jdtls](https://download.eclipse.org/jdtls/snapshots/)
- **Kotlin**: Download from [kotlin-language-server releases](https://github.com/fwcd/kotlin-language-server/releases)

## Usage

### Check LSP Server Availability

```bash
typeglass doctor
```

Verifies that required LSP servers are installed and accessible in PATH.

### Navigate Type Graph

```bash
# Start from a symbol
typeglass from --symbol TypeName --depth 2

# Start from a file
typeglass from --file src/main.rs --depth 1

# Start from a module
typeglass from --module src/domain --depth 3

# Analyze all public exports
typeglass from --public-exports --depth 1
```

### Output Formats

TypeGlass supports multiple output formats for different use cases:

```bash
# Human-readable (default)
typeglass from --symbol MyType

# JSON for programmatic processing
typeglass from --symbol MyType --format json

# Graphviz DOT for visualization
typeglass from --symbol MyType --format dot > graph.dot
dot -Tpng graph.dot -o graph.png

# Mermaid diagrams for documentation
typeglass from --symbol MyType --format mermaid > graph.mmd
```

### Clear Cache

```bash
# Clear cached graph data
typeglass gc
```

### Initialize Configuration

```bash
# Create .typeglass.toml config file
typeglass init
```

### Get Help

```bash
# General help
typeglass --help

# Command-specific help
typeglass from --help
```

## Example Output

### Human-readable format
```bash
$ typeglass from --symbol TypeGraph --depth 2
✅ Built graph from 'TypeGraph' with 15 nodes and 23 edges (depth: 2, language: Rust)
```

### JSON format
```bash
$ typeglass from --symbol TypeGraph --format json
```
```json
{
  "nodes": [
    {
      "name": "TypeGraph",
      "kind": "Struct",
      "location": { "file": "src/domain/graph.rs", "line": 42 }
    }
  ],
  "edges": [
    {
      "from": "TypeGraph",
      "to": "TypeNode",
      "kind": "Contains"
    },
    {
      "from": "MyStruct",
      "to": "MyTrait",
      "kind": "Extends"
    }
  ]
}
```

### DOT format
```bash
$ typeglass from --symbol TypeGraph --format dot
digraph TypeGraph {
  rankdir=TB;
  TypeGraph [label="TypeGraph", shape=box];
  TypeNode [label="TypeNode", shape=box];
  TypeGraph -> TypeNode [label="contains"];
}
```

## How It Works

1. **Language Detection**: Auto-detects project language (Rust/TypeScript/Go) from workspace markers
2. **LSP Initialization**: Starts and initializes LSP server for detected language
3. **Symbol Finding**: Uses `workspace/symbol` to locate type definitions
4. **Semantic Analysis**: Queries `textDocument/hover` to infer relationship types
5. **Edge Construction**: Builds TypeEdge with semantic EdgeKind (Extends, Implements, Instantiates, Contains)
6. **Reference Tracking**: Queries `textDocument/references` to find usages
7. **Recursive Traversal**: BFS traversal up to specified depth, preventing cycles
8. **Smart Caching**: Caches results with 5-minute TTL for faster subsequent queries

## Architecture

Follows hexagonal architecture with strict type-level domain modeling:

- **Domain**: Pure business logic with 40+ types
- **Application**: Use cases and command handlers
- **Infrastructure**: LSP client, cache, filesystem, CLI adapters
- **Principles**: Hexagonal ports & adapters, type-driven design

Key design decisions:
- **Lazy evaluation**: Build graph incrementally, not upfront
- **LSP-native**: Use language servers for accurate analysis
- **Semantic relationships**: Infer EdgeKind from hover responses
- **Multi-language**: Pluggable language support via LSP

See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## Development

```bash
# Build
cargo build

# Run tests (374 passing, >60% coverage)
cargo test

# Run clippy
cargo clippy --all-targets -- -D warnings

# Generate coverage report
cargo tarpaulin

# Run CLI
cargo run -- from --symbol TypeName --depth 2
```

## Testing

The project has comprehensive test coverage:
- 346+ unit tests for core logic
- 26 integration tests for CLI commands
- 12 error template tests
- 21 formatter tests

Run specific test suites:
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test cli_integration_test

# Formatter tests
cargo test formatters

# With coverage
cargo tarpaulin --out Html
```

## Roadmap

### ✅ Completed
- [x] Semantic EdgeKind detection (Extends, Implements, Instantiates)
- [x] Output formats (DOT, Mermaid)
- [x] Comprehensive error messages with next steps
- [x] CI/CD pipeline with GitHub Actions
- [x] Release automation

### 🚧 In Progress  
- [ ] TypeScript and Go integration tests
- [ ] Performance benchmarks
- [ ] Selective cache invalidation

### 📅 Planned
- [ ] Interactive exploration mode improvements
- [ ] VSCode extension
- [ ] Graph visualization server
- [ ] Query language for filtering

## Contributing

This project follows **GitFlow** for branch management:

- **`main`** - Production code, only merged from `release/*`
- **`develop`** - Integration branch for development  
- **`feature/*`** - New features
- **`release/*`** - Release preparation

See [GITFLOW.md](GITFLOW.md) for detailed workflow and CI/CD automation.

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch from `develop`
3. Follow [Conventional Commits](https://www.conventionalcommits.org/)
4. Add tests for new functionality
5. Ensure all tests pass and clippy is clean
6. Submit PR to `develop`

All PRs must pass CI checks (tests, clippy, coverage ≥ 60%) before merging.

See [RELEASING.md](RELEASING.md) for release process.

## Documentation

- [README.md](README.md): This file
- [ARCHITECTURE.md](ARCHITECTURE.md): Architecture deep dive
- [RELEASING.md](RELEASING.md): Release process
- [CHANGELOG.md](CHANGELOG.md): Version history
- [LSP_ISSUES.md](LSP_ISSUES.md): LSP integration notes (if exists)

## License

Dual licensed under MIT OR Apache-2.0.
