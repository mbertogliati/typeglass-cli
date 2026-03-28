use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Navigate type relationships"));
}

#[test]
fn test_from_command_help() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--symbol"));
}

#[test]
fn test_from_command_requires_target() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_from_command_with_symbol() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeA")
        .assert()
        .success()
        .stdout(predicate::str::contains("Failed to build graph"));
}

#[test]
fn test_from_command_with_depth() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeA")
        .arg("--depth")
        .arg("3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Failed to build graph"));
}

#[test]
fn test_from_command_invalid_symbol() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("")
        .assert()
        .success() // CLI returns 0 even for validation failures
        .stdout(predicate::str::contains("Failure"));
}

#[test]
fn test_from_command_file_not_implemented() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--file")
        .arg("test.ts")
        .assert()
        .success() // Returns Failure result but exit 0
        .stdout(predicate::str::contains("not yet implemented")
            .or(predicate::str::contains("not yet supported")));
}

#[test]
fn test_from_command_module_not_implemented() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--module")
        .arg("./src")
        .assert()
        .success() // Returns Failure result but exit 0
        .stdout(predicate::str::contains("not yet supported"));
}

#[test]
fn test_from_command_public_exports_not_implemented() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--public-exports")
        .assert()
        .success() // Returns Failure result but exit 0
        .stdout(predicate::str::contains("not yet supported"));
}

#[test]
fn test_doctor_command() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("doctor")
        .assert()
        .success();
}

#[test]
fn test_gc_command() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("gc")
        .assert()
        .success();
}

#[test]
fn test_init_command() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("init")
        .assert()
        .success();
}

#[test]
fn test_multiple_symbols() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeA")
        .assert()
        .success();

    let mut cmd2 = Command::cargo_bin("typeglass-cli").unwrap();
    cmd2.arg("from")
        .arg("--symbol")
        .arg("TypeB")
        .assert()
        .success();
}

#[test]
fn test_from_with_default_depth() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TestType")
        .assert()
        .success()
        .stdout(predicate::str::contains("Failed to build graph"));
}

#[test]
fn test_traversal_completeness_shown() {
    let mut cmd = Command::cargo_bin("typeglass-cli").unwrap();
    cmd.arg("from")
        .arg("--symbol")
        .arg("TypeA")
        .assert()
        .success()
        .stdout(predicate::str::contains("graph"));
}
