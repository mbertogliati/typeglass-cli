use crate::domain::contracts::{MachineCode, UserMessage};

use super::types::{
    Language, LanguageDetectionError, LanguageExtension, LanguageExtensionError, LanguageRuleset,
    LanguageRulesetError, ScanFileLimit, ScanFileLimitError,
};

impl Language {
    pub const fn supported() -> &'static [Language] {
        &[Language::TypeScript, Language::Rust, Language::Go]
    }
}

impl ScanFileLimit {
    pub fn new(value: usize) -> Result<Self, ScanFileLimitError> {
        if value == 0 {
            return Err(ScanFileLimitError::Zero);
        }
        Ok(Self { value })
    }
}

impl LanguageExtension {
    pub fn new(value: String) -> Result<Self, LanguageExtensionError> {
        if value.trim().is_empty() {
            return Err(LanguageExtensionError::Empty);
        }
        if !value.starts_with('.') {
            return Err(LanguageExtensionError::MissingDotPrefix { value });
        }
        Ok(Self { value })
    }
}

impl LanguageRuleset {
    pub fn validate(&self) -> Result<(), LanguageRulesetError> {
        if self.rules.is_empty() {
            return Err(LanguageRulesetError::Empty);
        }
        if self.rules.iter().any(|rule| rule.extensions.is_empty()) {
            return Err(LanguageRulesetError::RuleWithoutExtensions);
        }
        Ok(())
    }
}

impl UserMessage for LanguageDetectionError {
    fn user_message(&self) -> String {
        self.to_string()
    }
}

impl MachineCode for LanguageDetectionError {
    fn code(&self) -> &'static str {
        match self {
            LanguageDetectionError::NoSourceFilesFound { .. } => "LANG_NO_SOURCE_FILES",
            LanguageDetectionError::MultipleLanguagesDetected { .. } => "LANG_MULTIPLE_DETECTED",
            LanguageDetectionError::UnsupportedLanguage { .. } => "LANG_UNSUPPORTED",
            LanguageDetectionError::DirectoryNotReadable { .. } => "LANG_DIR_NOT_READABLE",
            LanguageDetectionError::TooManyFilesToScan { .. } => "LANG_SCAN_LIMIT",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::{LanguageRule, LanguageRuleset};

    #[test]
    fn scan_file_limit_cannot_be_zero() {
        let result = ScanFileLimit::new(0);
        assert!(matches!(result, Err(ScanFileLimitError::Zero)));
    }

    #[test]
    fn extension_requires_dot_prefix() {
        let result = LanguageExtension::new("ts".to_string());
        assert!(matches!(
            result,
            Err(LanguageExtensionError::MissingDotPrefix { .. })
        ));
    }

    #[test]
    fn ruleset_rejects_rule_without_extensions() {
        let ruleset = LanguageRuleset {
            rules: vec![LanguageRule {
                language: Language::TypeScript,
                extensions: vec![],
            }],
        };

        let result = ruleset.validate();
        assert!(matches!(
            result,
            Err(LanguageRulesetError::RuleWithoutExtensions)
        ));
    }
}
