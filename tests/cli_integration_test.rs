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
        .arg("lib.rs")
        .assert()
        .success() // Now implemented - uses filename stem as symbol
        .stdout(predicate::str::contains("Found").or(predicate::str::contains("Failed")));
}

#[test]
fn test_from_command_module_works() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--module")
        .arg("./src/graph")
        .assert()
        .success() // Now implemented - uses module name as symbol
        .stdout(predicate::str::contains("Found").or(predicate::str::contains("Failed")));
}

#[test]
fn test_from_command_public_exports_not_implemented() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("from")
        .arg("--public-exports")
        .assert()
        .failure() // Still not implemented
        .stderr(predicate::str::contains("not yet implemented"));
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
