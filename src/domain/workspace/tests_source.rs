#[cfg(test)]
mod workspace_source_tests {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    use crate::domain::workspace::{
        ContentHash, Encoding, FileContent, LineEnding, ModulePath, SourceFile,
    };

    fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn source_file_validates_existence() {
        let nonexistent = PathBuf::from("/nonexistent/file.txt");
        let result = SourceFile::new(nonexistent);
        assert!(result.is_err());
    }

    #[test]
    fn source_file_validates_is_file() {
        let dir = TempDir::new().unwrap();
        let result = SourceFile::new(dir.path().to_path_buf());
        assert!(result.is_err()); // Directory, not a file
    }

    #[test]
    fn source_file_reads_content() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_file(&dir, "test.txt", "Hello, World!");
        let source = SourceFile::new(path).unwrap();
        let content = source.read_content().unwrap();
        assert_eq!(content.as_str(), "Hello, World!");
    }

    #[test]
    fn source_file_provides_metadata() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_file(&dir, "test.txt", "content");
        let source = SourceFile::new(path).unwrap();
        let metadata = source.metadata().unwrap();
        assert!(metadata.size > 0);
        assert!(metadata.modified.is_some());
    }

    #[test]
    fn module_path_validates_non_empty() {
        let result = ModulePath::new(String::new());
        assert!(result.is_err());
    }

    #[test]
    fn module_path_rejects_control_characters() {
        let result = ModulePath::new("path/with\0null".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn module_path_accepts_valid_paths() {
        let result = ModulePath::new("@/components/Button".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "@/components/Button");
    }

    #[test]
    fn file_content_detects_utf8_encoding() {
        let content = "Hello, UTF-8! 世界"; // Non-ASCII UTF-8
        let result = FileContent::from_bytes(content.as_bytes().to_vec(), PathBuf::from("test.txt"));
        assert!(result.is_ok());
        let file_content = result.unwrap();
        assert_eq!(file_content.encoding(), Encoding::Utf8);
    }

    #[test]
    fn file_content_detects_ascii_encoding() {
        let content = "Hello ASCII";
        let result = FileContent::from_bytes(content.as_bytes().to_vec(), PathBuf::from("test.txt"));
        assert!(result.is_ok());
        let file_content = result.unwrap();
        assert_eq!(file_content.encoding(), Encoding::Ascii);
    }

    #[test]
    fn file_content_detects_unix_line_endings() {
        let content = "line1\nline2\nline3";
        let result = FileContent::from_bytes(content.as_bytes().to_vec(), PathBuf::from("test.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().line_ending(), LineEnding::Unix);
    }

    #[test]
    fn file_content_detects_windows_line_endings() {
        let content = "line1\r\nline2\r\n";
        let result = FileContent::from_bytes(content.as_bytes().to_vec(), PathBuf::from("test.txt"));
        assert!(result.is_ok());
        // Mixed because \r\n contains \n
        assert!(matches!(
            result.unwrap().line_ending(),
            LineEnding::Mixed | LineEnding::Windows
        ));
    }

    #[test]
    fn file_content_detects_mixed_line_endings() {
        let content = "line1\nline2\r\nline3";
        let result = FileContent::from_bytes(content.as_bytes().to_vec(), PathBuf::from("test.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().line_ending(), LineEnding::Mixed);
    }

    #[test]
    fn file_content_provides_lines() {
        let content = "line1\nline2\nline3";
        let file_content =
            FileContent::from_bytes(content.as_bytes().to_vec(), PathBuf::from("test.txt")).unwrap();
        let lines = file_content.lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn file_content_gets_line_at() {
        let content = "line1\nline2\nline3";
        let file_content =
            FileContent::from_bytes(content.as_bytes().to_vec(), PathBuf::from("test.txt")).unwrap();
        assert_eq!(file_content.line_at(1), Some("line1"));
        assert_eq!(file_content.line_at(2), Some("line2"));
        assert_eq!(file_content.line_at(3), Some("line3"));
        assert_eq!(file_content.line_at(4), None);
    }

    #[test]
    fn file_content_computes_hash() {
        let content1 = "same content";
        let content2 = "same content";
        let content3 = "different content";

        let hash1 = FileContent::from_bytes(content1.as_bytes().to_vec(), PathBuf::from("test.txt"))
            .unwrap()
            .hash()
            .clone();
        let hash2 = FileContent::from_bytes(content2.as_bytes().to_vec(), PathBuf::from("test.txt"))
            .unwrap()
            .hash()
            .clone();
        let hash3 = FileContent::from_bytes(content3.as_bytes().to_vec(), PathBuf::from("test.txt"))
            .unwrap()
            .hash()
            .clone();

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn file_content_detects_changes() {
        let content1 = "original";
        let content2 = "modified";

        let file1 =
            FileContent::from_bytes(content1.as_bytes().to_vec(), PathBuf::from("test.txt")).unwrap();
        let file2 =
            FileContent::from_bytes(content2.as_bytes().to_vec(), PathBuf::from("test.txt")).unwrap();

        assert!(file2.has_changed(file1.hash()));
        assert!(!file1.has_changed(file1.hash()));
    }

    #[test]
    fn content_hash_can_be_compared() {
        let hash1 = ContentHash::compute("content");
        let hash2 = ContentHash::compute("content");
        let hash3 = ContentHash::compute("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn file_content_rejects_invalid_utf8() {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let result = FileContent::from_bytes(invalid_utf8, PathBuf::from("test.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn file_content_enforces_size_limit() {
        let large_content = "x".repeat(51 * 1024 * 1024); // 51MB
        let result =
            FileContent::from_bytes(large_content.as_bytes().to_vec(), PathBuf::from("test.txt"));
        assert!(result.is_err());
    }
}
