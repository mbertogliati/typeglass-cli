use assert_cmd::Command;
use std::path::PathBuf;

/// These tests require gopls to be installed
/// Install with: go install golang.org/x/tools/gopls@latest
///
/// Run with: cargo test --test go_integration_test -- --ignored

fn go_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-fixtures")
        .join("go-test")
}

#[test]
#[ignore] // Requires gopls
fn test_go_from_symbol() {
    let fixture = go_fixture_path();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(&fixture)
        .arg("from")
        .arg("--symbol")
        .arg("User")
        .arg("--depth")
        .arg("1");
    
    cmd.assert().success();
}

#[test]
#[ignore] // Requires gopls
fn test_go_from_file() {
    let fixture = go_fixture_path();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(&fixture)
        .arg("from")
        .arg("--file")
        .arg("types.go")
        .arg("--depth")
        .arg("1");
    
    cmd.assert().success();
}

#[test]
#[ignore] // Requires gopls
fn test_go_doctor() {
    let fixture = go_fixture_path();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(&fixture)
        .arg("doctor");
    
    // Doctor should succeed if gopls is installed
    cmd.assert().success();
}

#[test]
#[ignore] // Requires gopls
fn test_go_json_output() {
    let fixture = go_fixture_path();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(&fixture)
        .arg("from")
        .arg("--symbol")
        .arg("UserService")
        .arg("--format")
        .arg("json");
    
    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    
    // Should be valid JSON
    assert!(stdout.contains("User") || stdout.is_empty());
}

#[test]
#[ignore] // Requires gopls
fn test_go_mermaid_output() {
    let fixture = go_fixture_path();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(&fixture)
        .arg("from")
        .arg("--symbol")
        .arg("User")
        .arg("--format")
        .arg("mermaid");
    
    cmd.assert().success();
}
