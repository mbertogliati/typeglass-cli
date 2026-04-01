# Performance Benchmarks

This document tracks performance benchmarks for TypeGlass CLI.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench graph_traversal

# Save baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

## Benchmark Suites

### Graph Queries

Measures performance of querying graph properties:
- `nodes`: Get all nodes count
- `edges`: Get all edges count

Tested with graph sizes: 10, 50, 100 nodes

### Graph Construction

Measures performance of building graphs:
- `linear`: Chain of N nodes (A -> B -> C -> ...)
- `diamond`: Diamond dependency pattern at D depth

Tested sizes:
- Linear: 10, 50, 100, 500 nodes
- Diamond: 3, 5, 7 levels deep

### EdgeKind Operations

Measures performance of EdgeKind operations:
- `clone`: Clone EdgeKind values
- `copy`: Copy EdgeKind values (should be faster due to Copy trait)

## Baseline Results

To be established on first run. Run benchmarks and record results here:

```bash
cargo bench > benchmark_results.txt
```

Expected performance characteristics:
- Graph queries should be O(1) - just returning collection size
- Linear construction should be O(n)
- Diamond construction should be O(2^depth)
- EdgeKind copy should be faster than clone (Copy trait)

## Performance Goals

- Graph construction: < 1ms for graphs up to 100 nodes
- Graph queries: < 10μs regardless of size
- EdgeKind operations: < 1ns per operation

## Optimization Opportunities

1. **Graph Storage**: Consider using indices instead of HashMap for faster lookups
2. **Edge Storage**: Vec might be faster than HashMap for small graphs
3. **Symbol Names**: Intern strings to reduce allocations
4. **Cache**: Profile cache hit/miss rates

## How to Profile

```bash
# CPU profiling with cargo-flamegraph
cargo install flamegraph
cargo flamegraph --bench graph_traversal

# Memory profiling with valgrind
valgrind --tool=massif --massif-out-file=massif.out \
    target/release/deps/graph_traversal-*
ms_print massif.out

# Heap profiling with dhat
cargo install dhat
# Add #[global_allocator] in benchmark
cargo bench --bench graph_traversal
```

## Continuous Monitoring

Benchmarks should be run:
- Before major refactorings
- When adding new features
- On release candidates
- Monthly on main branch

Store baseline results in git to track performance over time.
