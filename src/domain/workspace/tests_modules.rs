#[cfg(test)]
mod workspace_modules_tests {
    use std::path::PathBuf;

    use crate::domain::graph::SourceRange;
    use crate::domain::workspace::{
        DependencyKind, ExportKind, ExportStatement, ImportKind, ImportStatement, ModuleDependency,
        ModuleDependencyGraph, ModuleNode, ModulePath,
    };

    fn dummy_range() -> SourceRange {
        SourceRange::single_line(PathBuf::from("test.ts"), 1, 1, 10).unwrap()
    }

    #[test]
    fn import_named_identifies_correctly() {
        let import = ImportStatement {
            source: ModulePath::new("./module".to_string()).unwrap(),
            kind: ImportKind::Named {
                names: vec!["Foo".to_string()],
            },
            location: dummy_range(),
        };
        assert!(import.is_named());
        assert!(!import.is_default());
        assert!(!import.is_namespace());
        assert!(!import.is_side_effect());
    }

    #[test]
    fn import_extracts_names() {
        let import = ImportStatement {
            source: ModulePath::new("./module".to_string()).unwrap(),
            kind: ImportKind::Named {
                names: vec!["Foo".to_string(), "Bar".to_string()],
            },
            location: dummy_range(),
        };
        let names = import.imported_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Foo"));
        assert!(names.contains(&"Bar"));
    }

    #[test]
    fn import_side_effect_has_no_names() {
        let import = ImportStatement {
            source: ModulePath::new("./styles.css".to_string()).unwrap(),
            kind: ImportKind::SideEffect,
            location: dummy_range(),
        };
        assert!(import.imported_names().is_empty());
    }

    #[test]
    fn export_identifies_reexport() {
        let export = ExportStatement {
            kind: ExportKind::ReExport {
                from: ModulePath::new("./other".to_string()).unwrap(),
                names: vec!["Foo".to_string()],
            },
            location: dummy_range(),
        };
        assert!(export.is_reexport());
    }

    #[test]
    fn export_extracts_names() {
        let export = ExportStatement {
            kind: ExportKind::Named {
                names: vec!["Foo".to_string(), "Bar".to_string()],
            },
            location: dummy_range(),
        };
        let names = export.exported_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn export_reexport_provides_source() {
        let from = ModulePath::new("./source".to_string()).unwrap();
        let export = ExportStatement {
            kind: ExportKind::ReExportAll {
                from: from.clone(),
            },
            location: dummy_range(),
        };
        assert_eq!(export.reexport_source(), Some(&from));
    }

    #[test]
    fn module_dependency_identifies_kinds() {
        let dep = ModuleDependency {
            from: ModulePath::new("./a".to_string()).unwrap(),
            to: ModulePath::new("./b".to_string()).unwrap(),
            kind: DependencyKind::Direct,
            import_statements: vec![],
        };
        assert!(dep.is_direct());
        assert!(!dep.is_circular());
        assert!(!dep.is_external());
    }

    #[test]
    fn module_node_tracks_imports_exports() {
        let mut node =
            ModuleNode::new(ModulePath::new("./test".to_string()).unwrap(), PathBuf::from("test.ts"));

        assert_eq!(node.export_count(), 0);
        assert_eq!(node.import_count(), 0);

        node.add_import(ImportStatement {
            source: ModulePath::new("./other".to_string()).unwrap(),
            kind: ImportKind::Default {
                name: "Foo".to_string(),
            },
            location: dummy_range(),
        });

        node.add_export(ExportStatement {
            kind: ExportKind::Named {
                names: vec!["Bar".to_string()],
            },
            location: dummy_range(),
        });

        assert_eq!(node.import_count(), 1);
        assert_eq!(node.export_count(), 1);
    }

    #[test]
    fn module_node_lists_imported_modules() {
        let mut node =
            ModuleNode::new(ModulePath::new("./test".to_string()).unwrap(), PathBuf::from("test.ts"));

        let mod1 = ModulePath::new("./mod1".to_string()).unwrap();
        let mod2 = ModulePath::new("./mod2".to_string()).unwrap();

        node.add_import(ImportStatement {
            source: mod1.clone(),
            kind: ImportKind::SideEffect,
            location: dummy_range(),
        });

        node.add_import(ImportStatement {
            source: mod2.clone(),
            kind: ImportKind::SideEffect,
            location: dummy_range(),
        });

        let imported = node.imported_modules();
        assert_eq!(imported.len(), 2);
    }

    #[test]
    fn module_dependency_graph_tracks_counts() {
        let mut graph = ModuleDependencyGraph::new();
        assert_eq!(graph.module_count(), 0);
        assert_eq!(graph.dependency_count(), 0);

        graph.add_module(ModuleNode::new(
            ModulePath::new("./a".to_string()).unwrap(),
            PathBuf::from("a.ts"),
        ));

        graph.add_dependency(ModuleDependency {
            from: ModulePath::new("./a".to_string()).unwrap(),
            to: ModulePath::new("./b".to_string()).unwrap(),
            kind: DependencyKind::Direct,
            import_statements: vec![],
        });

        assert_eq!(graph.module_count(), 1);
        assert_eq!(graph.dependency_count(), 1);
    }

    #[test]
    fn module_dependency_graph_finds_modules() {
        let mut graph = ModuleDependencyGraph::new();
        let path = ModulePath::new("./test".to_string()).unwrap();

        graph.add_module(ModuleNode::new(path.clone(), PathBuf::from("test.ts")));

        assert!(graph.find_module(&path).is_some());
        assert!(graph
            .find_module(&ModulePath::new("./notfound".to_string()).unwrap())
            .is_none());
    }

    #[test]
    fn module_dependency_graph_queries_dependencies() {
        let mut graph = ModuleDependencyGraph::new();
        let a = ModulePath::new("./a".to_string()).unwrap();
        let b = ModulePath::new("./b".to_string()).unwrap();

        graph.add_dependency(ModuleDependency {
            from: a.clone(),
            to: b.clone(),
            kind: DependencyKind::Direct,
            import_statements: vec![],
        });

        let deps = graph.dependencies_of(&a);
        assert_eq!(deps.len(), 1);

        let dependents = graph.dependents_of(&b);
        assert_eq!(dependents.len(), 1);
    }

    #[test]
    fn module_dependency_graph_filters_circular() {
        let mut graph = ModuleDependencyGraph::new();

        graph.add_dependency(ModuleDependency {
            from: ModulePath::new("./a".to_string()).unwrap(),
            to: ModulePath::new("./b".to_string()).unwrap(),
            kind: DependencyKind::Circular,
            import_statements: vec![],
        });

        graph.add_dependency(ModuleDependency {
            from: ModulePath::new("./c".to_string()).unwrap(),
            to: ModulePath::new("./d".to_string()).unwrap(),
            kind: DependencyKind::Direct,
            import_statements: vec![],
        });

        let circular = graph.circular_dependencies();
        assert_eq!(circular.len(), 1);
    }

    #[test]
    fn module_dependency_graph_filters_external() {
        let mut graph = ModuleDependencyGraph::new();

        graph.add_dependency(ModuleDependency {
            from: ModulePath::new("./a".to_string()).unwrap(),
            to: ModulePath::new("react".to_string()).unwrap(),
            kind: DependencyKind::External,
            import_statements: vec![],
        });

        graph.add_dependency(ModuleDependency {
            from: ModulePath::new("./b".to_string()).unwrap(),
            to: ModulePath::new("./c".to_string()).unwrap(),
            kind: DependencyKind::Direct,
            import_statements: vec![],
        });

        let external = graph.external_dependencies();
        assert_eq!(external.len(), 1);
    }
}
