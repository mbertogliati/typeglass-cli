use crate::domain::graph::TypeGraph;

/// Common trait for graph formatters
pub trait GraphFormatter {
    /// Format the graph into the target representation
    fn format(&self, graph: &TypeGraph) -> Result<String, FormatterError>;
    
    /// File extension for this format (e.g., "dot", "mmd")
    fn file_extension(&self) -> &str;
    
    /// Human-readable name for this format
    fn name(&self) -> &str;
}

#[derive(Debug, thiserror::Error)]
pub enum FormatterError {
    #[error("Failed to format graph: {0}")]
    FormatError(String),
}

// Re-export formatters
mod dot;
mod mermaid;

pub use dot::DotFormatter;
pub use mermaid::MermaidFormatter;
