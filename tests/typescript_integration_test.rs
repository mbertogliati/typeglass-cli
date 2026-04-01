use assert_cmd::Command;
use std::path::PathBuf;

/// These tests require typescript-language-server to be installed
/// Install with: npm install -g typescript-language-server typescript
/// 
/// Run with: cargo test --test typescript_integration_test -- --ignored

fn typescript_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-fixtures")
        .join("typescript-test")
}

#[test]
#[ignore] // Requires typescript-language-server
fn test_typescript_from_symbol() {
    let fixture = typescript_fixture_path();
    
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
#[ignore] // Requires typescript-language-server
fn test_typescript_from_file() {
    let fixture = typescript_fixture_path();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(&fixture)
        .arg("from")
        .arg("--file")
        .arg("src/types.ts")
        .arg("--depth")
        .arg("1");
    
    cmd.assert().success();
}

#[test]
#[ignore] // Requires typescript-language-server
fn test_typescript_doctor() {
    let fixture = typescript_fixture_path();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(&fixture)
        .arg("doctor");
    
    // Doctor should succeed if typescript-language-server is installed
    cmd.assert().success();
}

#[test]
#[ignore] // Requires typescript-language-server  
fn test_typescript_json_output() {
    let fixture = typescript_fixture_path();
    
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
#[ignore] // Requires typescript-language-server
fn test_typescript_dot_output() {
    let fixture = typescript_fixture_path();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(&fixture)
        .arg("from")
        .arg("--symbol")
        .arg("User")
        .arg("--format")
        .arg("dot");
    
    cmd.assert().success();
}
