# typeglass

**Navigate type relationships lazily through LSP**

`typeglass` is a CLI tool that builds type dependency graphs on-demand using Language Server Protocol (LSP). Instead of analyzing entire codebases upfront, it queries LSP servers lazily to explore type relationships as you navigate.

## Features

- 🔍 **Lazy graph building** - Query type dependencies on-demand via LSP
- 🌐 **Multi-language support** - Works with any language that has an LSP server (Rust, TypeScript, Go, etc.)
- 📊 **Real edges** - Constructs TypeEdge connections via `textDocument/references`
- 🔄 **Recursive traversal** - BFS traversal with configurable depth
- 🎯 **Cycle detection** - Prevents infinite loops with visited tracking
- 💾 **Smart caching** - File-based cache with 5-minute TTL
- 🏥 **Health checks** - `doctor` command verifies LSP availability

## Installation

```bash
cargo build --release
# Binary available at target/release/typeglass
```

## Usage

### Check LSP Server Availability

```bash
typeglass doctor
```

Verifies that required LSP servers are installed (e.g., `rust-analyzer`, `typescript-language-server`).

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

### Get Help

```bash
# General help
typeglass --help

# Command-specific help
typeglass from --help
```

## Example Output

```bash
$ typeglass from --symbol TypeGraph --depth 2
```

```json
{
  "status": "Success",
  "summary": "Found 2 nodes and 20 edges from 'TypeGraph' (complete graph, language: Rust)"
}
```

## How It Works

1. **LSP Initialization**: Starts and initializes LSP server for detected language
2. **Symbol Finding**: Uses `workspace/symbol` to locate type definitions
3. **Reference Tracking**: Queries `textDocument/references` to find usages
4. **Edge Construction**: Builds TypeEdge connections between symbols
5. **Recursive Traversal**: BFS traversal up to specified depth, preventing cycles

## Architecture

Follows hexagonal architecture with strict type-level domain modeling:

- **Domain**: 34+ types encoding business rules
- **Ports**: Input/output interfaces for adapters
- **Infrastructure**: LSP, cache, and filesystem adapters
- **Principles**: Monomorfización (no `dyn`), Curry-Howard compliance, smart constructors

See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## Requirements

- Rust 1.70+
- LSP server for your language:
  - Rust: `rust-analyzer` (install with `rustup component add rust-analyzer`)
  - TypeScript: `typescript-language-server` (install with `npm install -g typescript-language-server`)
  - Go: `gopls` (install with `go install golang.org/x/tools/gopls@latest`)

## Development

```bash
# Build
cargo build

# Run tests (146 passing)
cargo test

# Run CLI
cargo run -- from --symbol TypeName --depth 2
```

## Current Limitations

- Symbol resolution uses file path heuristic (not 100% accurate)
- EdgeKind is simplified (always `Contains`, needs semantic analysis)
- Limited to 10 references per symbol (prevents explosion)
- Hover parsing not yet implemented (would improve accuracy)

## Roadmap

- [ ] Improve symbol resolution (parse hover responses)
- [ ] Semantic EdgeKind detection (Contains vs Extends vs Instantiates)
- [ ] Output formats (dot, mermaid, detailed JSON)
- [ ] TypeScript and Go LSP testing
- [ ] Interactive exploration mode
- [ ] Performance optimizations

## Documentation

- `CONTEXT.md`: Design conversation history
- `ARCHITECTURE.md`: Hexagonal architecture details
- `LSP_ISSUES.md`: LSP integration challenges and solutions

## License

MIT
