# LSP Integration Issues & Improvements

This document tracks issues discovered during LSP integration testing and their systematic resolution.

## Discovered Issues

### 🔴 Blocker (Must fix for basic functionality)

**LSP-001: rust-analyzer needs indexing time**
- **Problem**: After `initialize`, rust-analyzer returns "file not found" because it's still indexing
- **Impact**: All queries fail immediately after startup
- **Test**: `test_lsp_init_with_immediate_query`
- **Fix**: Add configurable wait OR poll for ready notification
- **Status**: Open

### 🟠 Critical (Significant functionality impact)

**LSP-002: SymbolFinder finds imports not definitions**
- **Problem**: Regex grep finds first occurrence (often import), not actual definition
- **Example**: Finds `TypeNode` in `graph_builder.rs` imports, not `model_graph.rs` definition
- **Test**: `test_symbol_finder_prioritizes_definitions`
- **Fix**: Use LSP `workspace/symbol` OR heuristic prioritization
- **Status**: Open

### 🟡 Major (Important for production use)

**LSP-003: File URI encoding may be incorrect**
- **Problem**: `format!("file://{}", path)` may not handle encoding properly
- **Test**: `test_file_uri_encoding`
- **Fix**: Use `url` crate for proper encoding
- **Status**: Open

**LSP-004: Error messages lack actionable context**
- **Problem**: Errors like "file not found" don't explain which file, why, or what to do
- **Test**: `test_error_message_clarity`
- **Fix**: Structured error context with file, LSP response, actions
- **Status**: Open

**LSP-005: No trace/debug logs for LSP communication**
- **Problem**: No visibility into requests/responses when debugging
- **Test**: `test_lsp_tracing`
- **Fix**: Add structured logging with `RUST_LOG`
- **Status**: Open

**LSP-007: No timeout on LSP requests**
- **Problem**: Requests can hang indefinitely
- **Test**: `test_lsp_request_timeout`
- **Fix**: Add `tokio::time::timeout` with configurable duration
- **Status**: Open

### 🔵 Minor (Nice to have)

**LSP-006: No retry logic for transient failures**
- **Problem**: Immediate failure on error -32603, could be transient
- **Test**: `test_lsp_retry_on_transient_errors`
- **Fix**: Exponential backoff for specific error codes
- **Status**: Open

**LSP-008: No health check after initialization**
- **Problem**: Assume LSP ready after `initialize`, should verify
- **Test**: `test_lsp_health_check`
- **Fix**: Lightweight request to verify responsiveness
- **Status**: Open

---

## Resolution Strategy

### Phase 1: Blockers (Must have)
1. LSP-001: Add wait/poll after initialize
2. LSP-002: Use workspace/symbol instead of grep

### Phase 2: Critical (Should have)
3. LSP-003: Proper URI encoding
4. LSP-004: Better error messages
5. LSP-005: Tracing/logging

### Phase 3: Nice-to-have
6. LSP-007: Timeouts
7. LSP-006: Retry logic
8. LSP-008: Health checks

---

## TDD Approach

For each issue:
1. ✅ Write failing test that reproduces the issue
2. ✅ Implement minimal fix
3. ✅ Verify test passes
4. ✅ Add integration test
5. ✅ Update documentation
6. ✅ Commit with reference to issue ID

