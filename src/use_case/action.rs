#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserAction {
    InitializeWorkspace,
    InspectTypeFromSymbol,
    InspectTypeFromFile,
    FindReferences,
    InvalidateFiles,
    StartDaemon,
    StopDaemon,
    GetDaemonStatus,
    RunDoctor,
}
