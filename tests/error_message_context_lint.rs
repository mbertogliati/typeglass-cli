use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn error_messages_with_payload_include_context_placeholders() {
    let mut files = Vec::new();
    collect_rs_files(Path::new("src"), &mut files).expect("must collect source files");

    let mut violations = Vec::new();

    for file in files {
        let content = fs::read_to_string(&file).expect("must read source file");
        let lines: Vec<&str> = content.lines().collect();

        for (index, line) in lines.iter().enumerate() {
            let Some(message) = extract_error_message(line) else {
                continue;
            };

            let Some(variant_index) = find_variant_line_index(&lines, index + 1) else {
                continue;
            };

            if variant_has_payload(&lines, variant_index) && !message.contains('{') {
                violations.push(format!(
                    "{}:{} -> error message lacks context placeholder: `{}`",
                    file.display(),
                    index + 1,
                    message
                ));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "Found error messages without contextual placeholders:\n{}",
            violations.join("\n")
        );
    }
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files)?;
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn extract_error_message(line: &str) -> Option<&str> {
    let start = line.find("#[error(\"")? + "#[error(\"".len();
    let rest = &line[start..];
    let end = rest.rfind("\")]")?;
    Some(&rest[..end])
}

fn find_variant_line_index(lines: &[&str], start: usize) -> Option<usize> {
    for (index, line) in lines.iter().enumerate().skip(start) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("#[") {
            continue;
        }
        return Some(index);
    }
    None
}

fn variant_has_payload(lines: &[&str], variant_start: usize) -> bool {
    for line in lines.iter().skip(variant_start).take(8) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("#[") {
            continue;
        }
        if trimmed.contains('{') || trimmed.contains('(') {
            return true;
        }
        if trimmed.ends_with(',') {
            return false;
        }
    }
    false
}
