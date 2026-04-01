use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraversalBoundary {
    Primitive,
    ExternalDependency,
    DepthLimit,
    AlreadyVisited,
    PolicyExcluded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircularDependencyPolicy {
    WarnAndContinue,
    FailFast,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraversalPolicy {
    pub skip_generated_files: bool,
    pub max_depth: u8,
    pub max_nodes: usize,
    pub max_edges: usize,
    pub generated_file_policy: GeneratedFilePolicy,
    pub external_boundary_policy: ExternalBoundaryPolicy,
    pub circular_dependency_policy: CircularDependencyPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryStopReason {
    Primitive,
    ExternalDependency,
    DepthLimit,
    CycleDetected,
    PolicyExcluded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceFileClassification {
    Domain,
    Generated,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedFilePattern {
    pub glob: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneratedFilePolicy {
    SkipByDefault,
    Include,
    SkipMatching(Vec<GeneratedFilePattern>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalBoundaryPolicy {
    StopAtExternal,
    IncludeExternalAsLeaf,
}
