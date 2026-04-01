# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Semantic EdgeKind inference from LSP hover responses
- DOT and Mermaid output formatters
- Comprehensive error message templates with actionable next steps
- GitHub Actions CI/CD pipeline
- Automated release workflow
- Code coverage tracking and enforcement (60% threshold)
- Security audit integration

### Changed
- EdgeKind now accurately reflects relationships (Extends, Instantiates, Variant, Contains)
- Error messages now include specific limitations and clear next steps

### Fixed
- Public exports test now correctly resolves src/lib.rs
- All clippy warnings resolved
- Invalid hover positions now handled gracefully

## [0.1.0] - Initial Release

### Added
- Lazy graph building via LSP
- Multi-language support (Rust, TypeScript, Go)
- BFS traversal with cycle detection
- File-based caching with TTL
- Doctor command for health checks
- From command with multiple target types
- GC command for cache management
- Init command for configuration
- Interactive REPL mode (WIP)

[Unreleased]: https://github.com/YOUR_USERNAME/typeglass-cli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/YOUR_USERNAME/typeglass-cli/releases/tag/v0.1.0
