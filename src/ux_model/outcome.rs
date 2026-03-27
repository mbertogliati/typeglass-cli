use serde::{Deserialize, Serialize};

use super::intent::{FromTarget, UserCommandKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserOutcome {
    Success(UserSuccess),
    Partial(UserPartial),
    Rejected(UserRejected),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserSuccess {
    DomainViewReady {
        target: FromTarget,
        depth: Option<u8>,
    },
    WorkspaceCacheCleaned,
    DoctorReportReady,
    WorkspaceInitialized,
    InteractiveSessionStarted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserSuccessKind {
    DomainViewReady,
    WorkspaceCacheCleaned,
    DoctorReportReady,
    WorkspaceInitialized,
    InteractiveSessionStarted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPartial {
    pub success: UserSuccess,
    pub warnings: Vec<UserWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserWarning {
    PartialCoverage { reason: String },
    ApproximateAnswer { reason: String },
    FollowupRecommended { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRejected {
    pub reason: UserRejectReason,
    pub next_step: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRejectReasonKind {
    AmbiguousRequest,
    InvalidRequest,
    FeatureNotReady,
    MissingContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRejectReason {
    AmbiguousRequest { message: String },
    InvalidRequest { message: String },
    FeatureNotReady { message: String },
    MissingContext { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserOutcomeContract {
    pub success: Vec<UserSuccessKind>,
    pub allows_partial: bool,
    pub rejections: Vec<UserRejectReasonKind>,
}

pub fn outcome_contract_for_command(command: UserCommandKind) -> UserOutcomeContract {
    let common_rejections = vec![
        UserRejectReasonKind::AmbiguousRequest,
        UserRejectReasonKind::InvalidRequest,
        UserRejectReasonKind::MissingContext,
        UserRejectReasonKind::FeatureNotReady,
    ];

    match command {
        UserCommandKind::From => UserOutcomeContract {
            success: vec![UserSuccessKind::DomainViewReady],
            allows_partial: true,
            rejections: common_rejections,
        },
        UserCommandKind::Gc => UserOutcomeContract {
            success: vec![UserSuccessKind::WorkspaceCacheCleaned],
            allows_partial: false,
            rejections: common_rejections,
        },
        UserCommandKind::Doctor => UserOutcomeContract {
            success: vec![UserSuccessKind::DoctorReportReady],
            allows_partial: false,
            rejections: common_rejections,
        },
        UserCommandKind::Init => UserOutcomeContract {
            success: vec![UserSuccessKind::WorkspaceInitialized],
            allows_partial: false,
            rejections: common_rejections,
        },
        UserCommandKind::Interactive => UserOutcomeContract {
            success: vec![UserSuccessKind::InteractiveSessionStarted],
            allows_partial: false,
            rejections: common_rejections,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_outcome_requires_next_step() {
        let rejected = UserRejected {
            reason: UserRejectReason::InvalidRequest {
                message: "Missing --symbol/--file/--module/--public-exports".to_string(),
            },
            next_step: "Pass exactly one target flag.".to_string(),
        };

        assert!(!rejected.next_step.trim().is_empty());
    }
}
