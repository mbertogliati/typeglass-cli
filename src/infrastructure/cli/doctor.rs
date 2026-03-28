#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticCategory {
    AutoRecoverable,
    UserActionRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticIssue {
    pub code: &'static str,
    pub severity: DiagnosticSeverity,
    pub category: DiagnosticCategory,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub issues: Vec<DiagnosticIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoctorCheck {
    BinaryVersion,
    RuntimeDependency,
    LspAvailability,
    WorkspaceAccessibility,
    CacheDirectoryWritable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorOutcome {
    pub check: DoctorCheck,
    pub status: DoctorStatus,
    pub issue: Option<DiagnosticIssue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Remediation {
    AutoFixable { action: AutoFixAction },
    UserActionRequired { instruction: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoFixAction {
    RestartDaemon,
    CleanupStaleState,
    RecreateCacheDirectory,
}
