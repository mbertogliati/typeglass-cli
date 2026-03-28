#[cfg(test)]
mod traversal_types_tests {
    use std::path::PathBuf;

    use crate::domain::graph::{
        CrossModuleReference, CrossModuleReferenceGraph, QualifiedSymbolName, ReferenceKind,
        ReferenceQuery, ReferenceResult, ResolutionStrategy, SourceLocation, SymbolName,
        SymbolReference, SymbolResolution, TraversalDirection, TraversalFilter, TraversalProgress,
    };

    #[test]
    fn traversal_direction_has_sensible_default() {
        let direction = TraversalDirection::default();
        assert_eq!(direction, TraversalDirection::Downstream);
    }

    #[test]
    fn traversal_filter_default_is_permissive() {
        let filter = TraversalFilter::default();
        assert!(filter.is_permissive());
    }

    #[test]
    fn traversal_progress_tracks_completion() {
        let mut progress = TraversalProgress::new(5);
        assert!(progress.is_active);
        assert!(!progress.is_complete());
        assert_eq!(progress.estimated_percent_complete, 0);

        progress.update(10, 20, 3);
        assert_eq!(progress.nodes_visited, 10);
        assert_eq!(progress.edges_followed, 20);
        assert!(progress.estimated_percent_complete > 0);

        progress.complete();
        assert!(progress.is_complete());
        assert_eq!(progress.estimated_percent_complete, 100);
    }

    #[test]
    fn symbol_resolution_variants_are_distinguishable() {
        let unique = SymbolResolution::Unique {
            qualified: QualifiedSymbolName {
                module_path: PathBuf::from("src/lib.rs"),
                symbol: SymbolName("Foo".to_string()),
            },
            location: SourceLocation {
                file: PathBuf::from("src/lib.rs"),
                line: 10,
                column: 1,
            },
        };
        assert!(unique.is_unique());
        assert_eq!(unique.candidate_count(), 1);

        let not_found = SymbolResolution::NotFound {
            searched_symbol: SymbolName("Bar".to_string()),
        };
        assert!(not_found.is_not_found());
        assert_eq!(not_found.candidate_count(), 0);
    }

    #[test]
    fn reference_result_filters_by_kind() {
        let result = ReferenceResult {
            query: ReferenceQuery::BySymbol {
                symbol: SymbolName("test".to_string()),
            },
            references: vec![
                SymbolReference {
                    symbol: SymbolName("test".to_string()),
                    location: SourceLocation {
                        file: PathBuf::from("a.rs"),
                        line: 1,
                        column: 1,
                    },
                    kind: ReferenceKind::Read,
                    context: None,
                },
                SymbolReference {
                    symbol: SymbolName("test".to_string()),
                    location: SourceLocation {
                        file: PathBuf::from("b.rs"),
                        line: 2,
                        column: 1,
                    },
                    kind: ReferenceKind::Call,
                    context: None,
                },
            ],
            is_complete: true,
            files_scanned: 2,
        };

        assert_eq!(result.reference_count(), 2);
        assert_eq!(result.references_by_kind(ReferenceKind::Read).len(), 1);
        assert_eq!(result.references_by_kind(ReferenceKind::Call).len(), 1);
        assert_eq!(result.unique_files().len(), 2);
    }

    #[test]
    fn cross_module_reference_graph_tracks_references() {
        let mut graph = CrossModuleReferenceGraph::new();
        assert_eq!(graph.reference_count(), 0);
        assert_eq!(graph.module_count(), 0);

        let reference = CrossModuleReference {
            source_module: PathBuf::from("src/a.rs"),
            target_module: PathBuf::from("src/b.rs"),
            symbol: SymbolName("Foo".to_string()),
            qualified_target: QualifiedSymbolName {
                module_path: PathBuf::from("src/b.rs"),
                symbol: SymbolName("Foo".to_string()),
            },
            reference_location: SourceLocation {
                file: PathBuf::from("src/a.rs"),
                line: 5,
                column: 10,
            },
            definition_location: SourceLocation {
                file: PathBuf::from("src/b.rs"),
                line: 1,
                column: 1,
            },
            import_path: Some("crate::b::Foo".to_string()),
            crosses_package_boundary: false,
        };

        graph.add_reference(reference);
        assert_eq!(graph.reference_count(), 1);
        assert_eq!(graph.module_count(), 2);
        assert_eq!(graph.internal_references().len(), 1);
        assert_eq!(graph.external_references().len(), 0);
    }

    #[test]
    fn cross_module_reference_knows_if_external() {
        let internal = CrossModuleReference {
            source_module: PathBuf::from("src/a.rs"),
            target_module: PathBuf::from("src/b.rs"),
            symbol: SymbolName("Foo".to_string()),
            qualified_target: QualifiedSymbolName {
                module_path: PathBuf::from("src/b.rs"),
                symbol: SymbolName("Foo".to_string()),
            },
            reference_location: SourceLocation {
                file: PathBuf::from("src/a.rs"),
                line: 1,
                column: 1,
            },
            definition_location: SourceLocation {
                file: PathBuf::from("src/b.rs"),
                line: 1,
                column: 1,
            },
            import_path: None,
            crosses_package_boundary: false,
        };
        
        assert!(internal.is_internal());
        assert!(!internal.is_external());
    }
}
