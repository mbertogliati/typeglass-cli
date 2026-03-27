#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEnvironment {
    Local,
    Ci,
    Container,
    NoTty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceAccessState {
    Accessible,
    Inaccessible,
}
