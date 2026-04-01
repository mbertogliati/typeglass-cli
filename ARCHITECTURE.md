# TypeGlass CLI - Hexagonal Architecture

## Overview

TypeGlass CLI follows **Hexagonal Architecture** (Ports & Adapters) principles:

- **Domain** contains business logic (pure, no infrastructure dependencies)
- **Application** orchestrates use cases (commands)
- **Infrastructure** provides adapters (LSP, filesystem, cache, CLI)
- **Ports** define interfaces between layers

## Directory Structure

```
src/
├── domain/                    # Core business logic (no external deps)
│   ├── graph/                # Type graph domain models
│   │   ├── model_graph.rs    # TypeNode, TypeGraph, TypeEdge
│   │   ├── model_symbols.rs  # SymbolName, QualifiedSymbolName
│   │   ├── model_traversal.rs # TraversalDirection, TraversalFilter
│   │   └── ...
│   ├── language/             # Language support
│   ├── workspace/            # Workspace models
│   ├── lsp/                  # LSP domain models
│   ├── recovery.rs           # Error recovery strategies
│   ├── performance.rs        # Performance constraints
│   └── ports/                # ⭐ Hexagonal ports
│       ├── in/               # Input ports (use case interfaces)
│       │   └── mod.rs        # Currently implicit
│       └── out/              # Output ports (infrastructure interfaces)
│           ├── lsp_gateway.rs         # LSP communication
│           ├── cache_repository.rs    # Cache persistence
│           └── filesystem_gateway.rs  # Filesystem operations
│
├── application/               # Use case orchestration
│   ├── service.rs            # Application service
│   └── use_cases/commands/   # Command handlers
│       ├── from.rs           # Main traversal command
│       ├── doctor.rs         # LSP diagnostic command
│       └── ...
│
├── infrastructure/            # ⭐ Adapters (organized by type)
│   ├── cli/                  # Primary adapter (entry point)
│   │   ├── args.rs           # CLI argument parsing
│   │   ├── impl.rs           # Command implementation
│   │   └── types.rs          # CLI types
│   ├── lsp/                  # LSP adapters
│   │   ├── client.rs         # LSP process manager
│   │   ├── init.rs           # LSP initialization
│   │   └── graph_builder.rs # Lazy graph construction
│   ├── persistence/          # Persistence adapters
│   │   └── cache/
│   │       └── file.rs       # File-based cache
│   └── filesystem/           # Filesystem adapters
│       └── scanner.rs        # Symbol finder
│
└── ux_model/                 # CLI-specific DTOs
    └── intent.rs             # User command models
```

## Key Principles

### 1. Monomorfización (No `dyn`)

✅ **Use generics with trait bounds:**
```rust
pub fn process<T: LspGateway>(gateway: &T) { ... }
```

❌ **Avoid trait objects:**
```rust
pub fn process(gateway: &dyn LspGateway) { ... }  // Don't use this
```

### 2. Dependency Direction

```
Infrastructure -> Application -> Domain
     ↓               ↓
  Ports (out)    Ports (in)
```

- Domain has **zero** infrastructure dependencies
- Application depends on **domain ports**, not concrete infrastructure
- Infrastructure implements **domain ports**

### 3. Port Design

**Output Ports** (domain/ports/out/):
- Traits defining what domain needs from infrastructure
- E.g., `LspGateway`, `CacheRepository`, `FilesystemGateway`

**Input Ports** (domain/ports/in/):
- Currently implicit (command structs serve as input ports)
- Could be extracted as traits if needed

### 4. Smart Constructors (Curry-Howard)

All domain types use smart constructors:
```rust
impl Position {
    pub fn new(line: u32, column: u32) -> Result<Self, PositionError> {
        if line == 0 { return Err(...); }
        Ok(Position { line, column })
    }
}
```

No public struct fields, no `unwrap`, no `as` casts.

## Test Organization

```
tests/
├── cli_integration_test.rs   # End-to-end CLI tests
└── (unit tests in modules)    # Domain unit tests
```

## Build & Test

```bash
# Build
cargo build

# Run tests
cargo test

# Run specific test
cargo test test_name

# Check for issues
cargo clippy
```

## Future Work

1. **Implement ports in infrastructure**
   - Make LspClient implement LspGateway
   - Make GraphCache implement CacheRepository
   - Make SymbolFinder implement FilesystemGateway

2. **Dependency injection**
   - Application uses port traits, not concrete types
   - Infrastructure injected via generics (not dyn)

3. **Port documentation**
   - Add examples to port traits
   - Document expected behavior contracts

## References

- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [Curry-Howard Correspondence](https://en.wikipedia.org/wiki/Curry%E2%80%93Howard_correspondence)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
