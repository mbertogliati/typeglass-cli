pub mod cache;
pub mod contracts;
pub mod daemon;
pub mod graph;
pub mod interactive;
pub mod language;
pub mod lsp;
pub mod performance;
pub mod ports;
pub mod recovery;
pub mod session;
pub mod telemetry;
pub mod workspace;

#[cfg(test)]
mod tests_performance;
#[cfg(test)]
mod tests_recovery;
