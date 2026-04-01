use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use std::path::PathBuf;

use typeglass_cli::domain::graph::{
    TypeGraph, TypeNode, TypeEdge, EdgeKind, SymbolName, SymbolKind,
    QualifiedSymbolName, SymbolOrigin, SourceLocation, TraversalDirection,
};
use typeglass_cli::domain::language::Language;

fn create_test_node(name: &str) -> TypeNode {
    TypeNode {
        id: QualifiedSymbolName {
            module_path: PathBuf::from("bench.rs"),
            symbol: SymbolName(name.to_string()),
        },
        name: SymbolName(name.to_string()),
        kind: SymbolKind::Struct,
        origin: SymbolOrigin::Canonical,
        location: SourceLocation {
            file: PathBuf::from("bench.rs"),
            line: 0,
            column: 0,
        },
        language: Language::Rust,
        generic_parameters: vec![],
    }
}

fn create_linear_graph(size: usize) -> TypeGraph {
    let mut graph = TypeGraph::empty();
    
    for i in 0..size {
        graph.add_node(create_test_node(&format!("Node{}", i)));
    }
    
    for i in 0..size - 1 {
        graph.add_edge(TypeEdge {
            from: SymbolName(format!("Node{}", i)),
            to: SymbolName(format!("Node{}", i + 1)),
            kind: EdgeKind::Contains,
        });
    }
    
    graph
}

fn create_diamond_graph(depth: usize) -> TypeGraph {
    let mut graph = TypeGraph::empty();
    
    // Root node
    graph.add_node(create_test_node("Root"));
    
    // Create diamond pattern at each level
    for level in 0..depth {
        let left = format!("Left{}", level);
        let right = format!("Right{}", level);
        let merge = format!("Merge{}", level);
        
        graph.add_node(create_test_node(&left));
        graph.add_node(create_test_node(&right));
        graph.add_node(create_test_node(&merge));
        
        if level == 0 {
            graph.add_edge(TypeEdge {
                from: SymbolName("Root".to_string()),
                to: SymbolName(left.clone()),
                kind: EdgeKind::Contains,
            });
            graph.add_edge(TypeEdge {
                from: SymbolName("Root".to_string()),
                to: SymbolName(right.clone()),
                kind: EdgeKind::Contains,
            });
        } else {
            let prev_merge = format!("Merge{}", level - 1);
            graph.add_edge(TypeEdge {
                from: SymbolName(prev_merge.clone()),
                to: SymbolName(left.clone()),
                kind: EdgeKind::Contains,
            });
            graph.add_edge(TypeEdge {
                from: SymbolName(prev_merge),
                to: SymbolName(right.clone()),
                kind: EdgeKind::Contains,
            });
        }
        
        graph.add_edge(TypeEdge {
            from: SymbolName(left),
            to: SymbolName(merge.clone()),
            kind: EdgeKind::Contains,
        });
        graph.add_edge(TypeEdge {
            from: SymbolName(right),
            to: SymbolName(merge),
            kind: EdgeKind::Contains,
        });
    }
    
    graph
}

fn bench_graph_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_queries");
    
    for size in [10, 50, 100].iter() {
        let graph = create_linear_graph(*size);
        
        group.bench_with_input(BenchmarkId::new("nodes", size), size, |b, _| {
            b.iter(|| {
                black_box(graph.nodes().len())
            });
        });
        
        group.bench_with_input(BenchmarkId::new("edges", size), size, |b, _| {
            b.iter(|| {
                black_box(graph.edges().len())
            });
        });
    }
    
    group.finish();
}

fn bench_graph_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_construction");
    
    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("linear", size), size, |b, &s| {
            b.iter(|| {
                black_box(create_linear_graph(s))
            });
        });
    }
    
    for depth in [3, 5, 7].iter() {
        group.bench_with_input(BenchmarkId::new("diamond", depth), depth, |b, &d| {
            b.iter(|| {
                black_box(create_diamond_graph(d))
            });
        });
    }
    
    group.finish();
}

fn bench_edge_kind_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_kind");
    
    let kinds = vec![
        EdgeKind::Contains,
        EdgeKind::Extends,
        EdgeKind::Instantiates,
        EdgeKind::Variant,
    ];
    
    group.bench_function("clone", |b| {
        b.iter(|| {
            for kind in &kinds {
                black_box(kind.clone());
            }
        });
    });
    
    group.bench_function("copy", |b| {
        b.iter(|| {
            for kind in &kinds {
                black_box(*kind);
            }
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_graph_queries, bench_graph_construction, bench_edge_kind_operations);
criterion_main!(benches);
