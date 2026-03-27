use thiserror::Error;

use crate::domain::language::Language;
use crate::domain::lsp::LanguageFeature;

use super::action::UserAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSupport {
    pub action: UserAction,
    pub language: Language,
    pub required_features: Vec<LanguageFeature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSupportMatrix {
    pub entries: Vec<ActionSupport>,
}

#[derive(Debug, Error)]
pub enum ActionSupportMatrixError {
    #[error("Action support matrix cannot be empty")]
    Empty,
    #[error("Action support matrix contains duplicated action/language entries")]
    DuplicatedEntry,
}

impl ActionSupportMatrix {
    pub fn validate(&self) -> Result<(), ActionSupportMatrixError> {
        if self.entries.is_empty() {
            return Err(ActionSupportMatrixError::Empty);
        }

        for (index, current) in self.entries.iter().enumerate() {
            if self
                .entries
                .iter()
                .skip(index + 1)
                .any(|other| other.action == current.action && other.language == current.language)
            {
                return Err(ActionSupportMatrixError::DuplicatedEntry);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_matrix_rejects_duplicates() {
        let entry = ActionSupport {
            action: UserAction::InspectTypeFromSymbol,
            language: Language::Rust,
            required_features: vec![LanguageFeature::TypeGraphTraversal],
        };

        let matrix = ActionSupportMatrix {
            entries: vec![entry.clone(), entry],
        };

        let result = matrix.validate();
        assert!(matches!(
            result,
            Err(ActionSupportMatrixError::DuplicatedEntry)
        ));
    }
}
