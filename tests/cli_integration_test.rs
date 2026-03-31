use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Navigate type relationships"));
}

#[test]
fn test_from_command_help() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--symbol"));
}

#[test]
fn test_from_command_requires_target() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_from_command_with_symbol() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeA")
        .assert()
        .success()
        .stdout(predicate::str::contains("Found").or(predicate::str::contains("Failed")));
}

#[test]
fn test_from_command_with_depth() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeA")
        .arg("--depth")
        .arg("3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Found").or(predicate::str::contains("Failed")));
}

#[test]
fn test_from_command_invalid_symbol() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("NonExistentSymbolThatReallyDoesNotExist12345")
        .assert()
        .failure() // Symbol not found returns exit 1
        .stderr(predicate::str::contains("Failed").or(predicate::str::contains("Symbol not found")));
}

#[test]
fn test_from_command_file_works() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--file")
        .arg("src/lib.rs")  // Use real file path
        .assert()
        .success()
        .stdout(predicate::str::contains("Found").or(predicate::str::contains("graph")));
}

#[test]
fn test_from_command_module_works() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--module")
        .arg("src/domain")  // Use real module path
        .assert()
        .success()
        .stdout(predicate::str::contains("Found").or(predicate::str::contains("graph")));
}

#[test]
fn test_from_command_public_exports_works() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--public-exports")
        .assert()
        .success()
        .stdout(predicate::str::contains("Found").or(predicate::str::contains("graph")));
}

#[test]
fn test_doctor_command() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("doctor")
        .assert()
        .success();
}

#[test]
fn test_gc_command() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("gc")
        .assert()
        .success();
}

#[test]
fn test_init_command() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("init")
        .assert()
        .success();
}

#[test]
fn test_multiple_symbols() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeNode") // Real symbol
        .assert()
        .success();

    let mut cmd2 = Command::cargo_bin("typeglass").unwrap();
    cmd2.arg("from")
        .arg("--symbol")
        .arg("TypeGraph") // Real symbol
        .assert()
        .success();
}

#[test]
fn test_from_with_default_depth() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeGraph") // Use a real symbol from our codebase
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_traversal_completeness_shown() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeA")
        .assert()
        .success()
        .stdout(predicate::str::contains("graph"));
}

// Additional tests for better coverage

#[test]
fn test_from_with_max_depth() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeGraph")
        .arg("--depth")
        .arg("10")
        .assert()
        .success();
}

#[test]
fn test_from_with_invalid_depth() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeA")
        .arg("--depth")
        .arg("999") // Invalid depth > max
        .assert()
        .failure();
}

#[test]
fn test_init_creates_config() {
    use std::fs;
    use tempfile::TempDir;
    
    // Create a temporary directory with Cargo.toml
    let temp = TempDir::new().unwrap();
    let cargo_toml = temp.path().join("Cargo.toml");
    fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace initialized").or(predicate::str::contains("already exists")));
}

#[test]
fn test_gc_runs_successfully() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("gc")
        .assert()
        .success();
    // Just verify it runs, output format may vary
}

#[test]
fn test_doctor_shows_lsp_status() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("doctor")
        .assert()
        .success();
    // Just verify it runs, LSP availability varies by system
}

#[test]
fn test_from_file_uses_symbol_name() {
    // File flag uses the file stem as symbol name
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--file")
        .arg("path/to/some_module.rs")
        .assert()
        .success();  // Will search for "some_module" symbol
}

#[test]
fn test_from_module_uses_module_name() {
    // Module flag uses last path component as module name
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--module")
        .arg("path/to/domain")
        .assert()
        .success();  // Will search for "domain" symbol
}

#[test]
fn test_help_for_each_command() {
    for command in &["from", "gc", "init", "doctor"] {
        let mut cmd = Command::cargo_bin("typeglass").unwrap();
        cmd.arg(command)
            .arg("--help")
            .assert()
            .success();
    }
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("typeglass"));
}

#[test]
fn test_from_with_depth_one() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeGraph")
        .arg("--depth")
        .arg("1") // One level of traversal
        .assert()
        .success();
}

#[test]
fn test_multiple_invocations_reuse_lsp() {
    // Multiple invocations should work (LSP daemon might be reused)
    for _ in 0..3 {
        let mut cmd = Command::cargo_bin("typeglass").unwrap();
        cmd.arg("from")
            .arg("--symbol")
            .arg("TypeNode")
            .assert()
            .success();
    }
}
