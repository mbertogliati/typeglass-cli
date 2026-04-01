use thiserror::Error;

use super::model::SymbolName;

#[derive(Debug, Error)]
pub enum SymbolNameError {
    #[error("Symbol name cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum SourceLocationError {
    #[error("Source location line must be greater than zero")]
    InvalidLine,
    #[error("Source location column must be greater than zero")]
    InvalidColumn,
}

#[derive(Debug, Error)]
pub enum GeneratedFilePatternError {
    #[error("Generated file pattern cannot be empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum TraversalPolicyError {
    #[error("Traversal max depth must be greater than zero")]
    ZeroDepth,
    #[error("Traversal max nodes must be greater than zero")]
    ZeroNodes,
    #[error("Traversal max edges must be greater than zero")]
    ZeroEdges,
}

#[derive(Debug, Error)]
pub enum GraphIntegrityError {
    #[error("Graph contains duplicate symbol node: {symbol:?}")]
    DuplicateNode { symbol: SymbolName },
    #[error("Graph edge points to missing node. From: {from:?}, To: {to:?}")]
    EdgeToMissingNode { from: SymbolName, to: SymbolName },
}
