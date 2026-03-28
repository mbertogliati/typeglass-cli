// Legacy contracts (to be refactored)
pub mod r#impl;
pub mod types;
pub use types::*;

// Hexagonal architecture ports
pub mod r#in;
pub mod out;
