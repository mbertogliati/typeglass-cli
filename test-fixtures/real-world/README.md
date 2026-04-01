# Real-World Test Fixtures

This directory contains real open-source projects used for stress testing TypeGlass CLI.

## Purpose

- Test with production codebases (not toy examples)
- Validate performance at scale (10k+ LOC)
- Ensure LSP integration works with complex type hierarchies
- Measure cache effectiveness

## Projects

### TypeScript
- **axios** - Popular HTTP client (~100k stars)
- **express** - Web framework (~65k stars)

### Go
- **cobra** - CLI framework (~38k stars)
- **gin** - Web framework (~78k stars)

### Java
- **spring-petclinic** - Spring Boot reference application

### Kotlin
- **ktor-samples** - Ktor framework samples

## Usage

These projects are cloned as git submodules or downloaded on-demand.
Tests are marked as `#[ignore]` by default and require LSP servers installed.

Run real-world tests:
```bash
cargo test --test real_world_tests -- --ignored
```

## Note

Real-world tests are NOT run in CI by default because:
1. Require external LSP servers installed
2. Take longer to execute
3. External dependencies (git clone)

They are meant for local validation and benchmarking.
