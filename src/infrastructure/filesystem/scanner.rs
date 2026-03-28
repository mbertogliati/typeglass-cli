use std::path::{Path, PathBuf};
use std::fs;
use regex::Regex;

/// Symbol finder - scans workspace for symbol occurrences
pub struct SymbolFinder {
    workspace_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SymbolLocation {
    pub file_path: PathBuf,
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum SymbolFinderError {
    #[error("Symbol not found: {symbol}")]
    NotFound { symbol: String },
    
    #[error("Multiple occurrences found for symbol: {symbol}")]
    Ambiguous { symbol: String, locations: Vec<SymbolLocation> },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl SymbolFinder {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Find first occurrence of symbol in workspace
    pub fn find_symbol(&self, symbol_name: &str) -> Result<SymbolLocation, SymbolFinderError> {
        let locations = self.find_all_occurrences(symbol_name)?;
        
        if locations.is_empty() {
            return Err(SymbolFinderError::NotFound {
                symbol: symbol_name.to_string(),
            });
        }

        // Return first occurrence (could be improved with ranking)
        Ok(locations[0].clone())
    }

    /// Find all occurrences of symbol
    fn find_all_occurrences(&self, symbol_name: &str) -> Result<Vec<SymbolLocation>, SymbolFinderError> {
        let mut locations = Vec::new();
        
        // Pattern: word boundary + symbol + word boundary
        let pattern = format!(r"\b{}\b", regex::escape(symbol_name));
        let re = Regex::new(&pattern).unwrap();

        self.scan_directory(&self.workspace_root, &re, symbol_name, &mut locations)?;

        Ok(locations)
    }

    fn scan_directory(
        &self,
        dir: &Path,
        pattern: &Regex,
        symbol_name: &str,
        locations: &mut Vec<SymbolLocation>,
    ) -> Result<(), SymbolFinderError> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden files and common ignore directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') 
                    || name == "target" 
                    || name == "node_modules" 
                    || name == "dist"
                    || name == "build" {
                    continue;
                }
            }

            if path.is_dir() {
                self.scan_directory(&path, pattern, symbol_name, locations)?;
            } else if self.is_source_file(&path) {
                self.scan_file(&path, pattern, symbol_name, locations)?;
            }
        }

        Ok(())
    }

    fn scan_file(
        &self,
        file_path: &Path,
        pattern: &Regex,
        _symbol_name: &str,
        locations: &mut Vec<SymbolLocation>,
    ) -> Result<(), SymbolFinderError> {
        let content = fs::read_to_string(file_path)?;

        for (line_num, line) in content.lines().enumerate() {
            if let Some(mat) = pattern.find(line) {
                locations.push(SymbolLocation {
                    file_path: file_path.to_path_buf(),
                    line: line_num as u32,
                    character: mat.start() as u32,
                });
            }
        }

        Ok(())
    }

    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext, "rs" | "ts" | "tsx" | "js" | "jsx" | "go")
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_find_symbol_in_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "struct TypeA {{}}").unwrap();
        writeln!(file, "struct TypeB {{}}").unwrap();

        let finder = SymbolFinder::new(temp_dir.path().to_path_buf());
        let location = finder.find_symbol("TypeA").unwrap();

        assert_eq!(location.line, 0);
        assert!(location.file_path.ends_with("test.rs"));
    }

    #[test]
    fn test_symbol_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let finder = SymbolFinder::new(temp_dir.path().to_path_buf());
        
        let result = finder.find_symbol("NonExistent");
        assert!(matches!(result, Err(SymbolFinderError::NotFound { .. })));
    }

    #[test]
    fn test_ignore_hidden_files() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create hidden file
        let hidden_path = temp_dir.path().join(".hidden.rs");
        let mut file = File::create(&hidden_path).unwrap();
        writeln!(file, "struct TypeA {{}}").unwrap();

        // Create normal file
        let normal_path = temp_dir.path().join("normal.rs");
        let mut file = File::create(&normal_path).unwrap();
        writeln!(file, "struct TypeB {{}}").unwrap();

        let finder = SymbolFinder::new(temp_dir.path().to_path_buf());
        
        // Should not find TypeA (in hidden file)
        assert!(finder.find_symbol("TypeA").is_err());
        
        // Should find TypeB (in normal file)
        assert!(finder.find_symbol("TypeB").is_ok());
    }

    #[test]
    fn test_word_boundary_matching() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "struct TypeA {{}}").unwrap();
        writeln!(file, "struct TypeAB {{}}").unwrap();
        writeln!(file, "struct BTypeA {{}}").unwrap();

        let finder = SymbolFinder::new(temp_dir.path().to_path_buf());
        let location = finder.find_symbol("TypeA").unwrap();

        // Should match exact "TypeA", not "TypeAB" or "BTypeA"
        assert_eq!(location.line, 0);
    }
}
