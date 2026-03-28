use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::source::ModulePath;
use crate::domain::graph::SourceRange;

/// Kind of import statement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    /// Named imports: import { A, B } from "module"
    Named { names: Vec<String> },
    /// Default import: import X from "module"
    Default { name: String },
    /// Namespace import: import * as X from "module"
    Namespace { alias: String },
    /// Side-effect import: import "module"
    SideEffect,
}

/// Import statement in source code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportStatement {
    /// Module being imported from
    pub source: ModulePath,
    /// Kind of import
    pub kind: ImportKind,
    /// Location in source file
    pub location: SourceRange,
}

impl ImportStatement {
    pub fn is_named(&self) -> bool {
        matches!(self.kind, ImportKind::Named { .. })
    }

    pub fn is_default(&self) -> bool {
        matches!(self.kind, ImportKind::Default { .. })
    }

    pub fn is_namespace(&self) -> bool {
        matches!(self.kind, ImportKind::Namespace { .. })
    }

    pub fn is_side_effect(&self) -> bool {
        matches!(self.kind, ImportKind::SideEffect)
    }

    pub fn imported_names(&self) -> Vec<&str> {
        match &self.kind {
            ImportKind::Named { names } => names.iter().map(|s| s.as_str()).collect(),
            ImportKind::Default { name } => vec![name.as_str()],
            ImportKind::Namespace { alias } => vec![alias.as_str()],
            ImportKind::SideEffect => vec![],
        }
    }
}

/// Kind of export statement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportKind {
    /// Named exports: export { A, B }
    Named { names: Vec<String> },
    /// Default export: export default X
    Default { name: String },
    /// Re-export: export { A } from "module"
    ReExport { from: ModulePath, names: Vec<String> },
    /// Re-export all: export * from "module"
    ReExportAll { from: ModulePath },
}

/// Export statement in source code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportStatement {
    /// Kind of export
    pub kind: ExportKind,
    /// Location in source file
    pub location: SourceRange,
}

impl ExportStatement {
    pub fn is_reexport(&self) -> bool {
        matches!(
            self.kind,
            ExportKind::ReExport { .. } | ExportKind::ReExportAll { .. }
        )
    }

    pub fn exported_names(&self) -> Vec<&str> {
        match &self.kind {
            ExportKind::Named { names } => names.iter().map(|s| s.as_str()).collect(),
            ExportKind::Default { name } => vec![name.as_str()],
            ExportKind::ReExport { names, .. } => names.iter().map(|s| s.as_str()).collect(),
            ExportKind::ReExportAll { .. } => vec![],
        }
    }

    pub fn reexport_source(&self) -> Option<&ModulePath> {
        match &self.kind {
            ExportKind::ReExport { from, .. } | ExportKind::ReExportAll { from } => Some(from),
            _ => None,
        }
    }
}

/// Module-level dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDependency {
    /// Source module
    pub from: ModulePath,
    /// Target module
    pub to: ModulePath,
    /// Kind of dependency
    pub kind: DependencyKind,
    /// Import statements that create this dependency
    pub import_statements: Vec<ImportStatement>,
}

/// Kind of module dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    /// Direct import
    Direct,
    /// Transitive (A imports B, B imports C => A transitively depends on C)
    Transitive,
    /// Circular dependency detected
    Circular,
    /// Crosses package boundary (external dependency)
    External,
}

impl ModuleDependency {
    pub fn is_direct(&self) -> bool {
        matches!(self.kind, DependencyKind::Direct)
    }

    pub fn is_circular(&self) -> bool {
        matches!(self.kind, DependencyKind::Circular)
    }

    pub fn is_external(&self) -> bool {
        matches!(self.kind, DependencyKind::External)
    }
}

/// Node in the module dependency graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleNode {
    /// Module path
    pub path: ModulePath,
    /// Source file path
    pub file: PathBuf,
    /// Exports from this module
    pub exports: Vec<ExportStatement>,
    /// Imports in this module
    pub imports: Vec<ImportStatement>,
}

impl ModuleNode {
    pub fn new(path: ModulePath, file: PathBuf) -> Self {
        Self {
            path,
            file,
            exports: Vec::new(),
            imports: Vec::new(),
        }
    }

    pub fn add_export(&mut self, export: ExportStatement) {
        self.exports.push(export);
    }

    pub fn add_import(&mut self, import: ImportStatement) {
        self.imports.push(import);
    }

    pub fn export_count(&self) -> usize {
        self.exports.len()
    }

    pub fn import_count(&self) -> usize {
        self.imports.len()
    }

    pub fn imported_modules(&self) -> Vec<&ModulePath> {
        self.imports.iter().map(|i| &i.source).collect()
    }
}

/// Module dependency graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDependencyGraph {
    /// All modules in the graph
    pub modules: Vec<ModuleNode>,
    /// Dependencies between modules
    pub dependencies: Vec<ModuleDependency>,
}

impl ModuleDependencyGraph {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: ModuleNode) {
        self.modules.push(module);
    }

    pub fn add_dependency(&mut self, dependency: ModuleDependency) {
        self.dependencies.push(dependency);
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }

    pub fn find_module(&self, path: &ModulePath) -> Option<&ModuleNode> {
        self.modules.iter().find(|m| m.path == *path)
    }

    pub fn dependencies_of(&self, module: &ModulePath) -> Vec<&ModuleDependency> {
        self.dependencies
            .iter()
            .filter(|d| d.from == *module)
            .collect()
    }

    pub fn dependents_of(&self, module: &ModulePath) -> Vec<&ModuleDependency> {
        self.dependencies
            .iter()
            .filter(|d| d.to == *module)
            .collect()
    }

    pub fn circular_dependencies(&self) -> Vec<&ModuleDependency> {
        self.dependencies.iter().filter(|d| d.is_circular()).collect()
    }

    pub fn external_dependencies(&self) -> Vec<&ModuleDependency> {
        self.dependencies.iter().filter(|d| d.is_external()).collect()
    }
}

impl Default for ModuleDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}
