use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test that doctor command checks all 5 languages
#[test]
fn test_doctor_checks_all_languages() {
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.arg("doctor");
    
    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    
    // Verify all 5 languages are checked
    assert!(stdout.contains("TypeScript"), "Should check TypeScript");
    assert!(stdout.contains("Rust"), "Should check Rust");
    assert!(stdout.contains("Go"), "Should check Go");
    assert!(stdout.contains("Java"), "Should check Java");
    assert!(stdout.contains("Kotlin"), "Should check Kotlin");
    
    // Verify LSP server names
    assert!(stdout.contains("typescript-language-server"));
    assert!(stdout.contains("rust-analyzer"));
    assert!(stdout.contains("gopls"));
    assert!(stdout.contains("jdtls"));
    assert!(stdout.contains("kotlin-language-server"));
}

/// Test Java project detection with pom.xml
#[test]
fn test_detect_java_via_pom_xml() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    fs::write(
        project.join("pom.xml"),
        r#"<?xml version="1.0"?>
<project>
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.test</groupId>
    <artifactId>test</artifactId>
    <version>1.0</version>
</project>
"#,
    )
    .unwrap();
    
    fs::create_dir_all(project.join("src/main/java/com/test")).unwrap();
    fs::write(
        project.join("src/main/java/com/test/Main.java"),
        "package com.test;\npublic class Main { }",
    )
    .unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("Main")
        .arg("--depth")
        .arg("1");
    
    // Should fail because jdtls not installed, but error should mention Java/jdtls
    let output = cmd.assert().failure();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).unwrap();
    
    // Should detect it's a Java project
    assert!(
        stderr.contains("jdtls") || stderr.contains("Java"),
        "Error should mention Java or jdtls. Got: {}",
        stderr
    );
}

/// Test Java project detection with build.gradle
#[test]
fn test_detect_java_via_gradle() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    fs::write(
        project.join("build.gradle"),
        r#"
plugins {
    id 'java'
}
group = 'com.test'
version = '1.0'
"#,
    )
    .unwrap();
    
    fs::create_dir_all(project.join("src/main/java/com/test")).unwrap();
    fs::write(
        project.join("src/main/java/com/test/App.java"),
        "package com.test;\npublic class App { }",
    )
    .unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("App")
        .arg("--depth")
        .arg("1");
    
    let output = cmd.assert().failure();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).unwrap();
    
    assert!(
        stderr.contains("jdtls") || stderr.contains("Java"),
        "Should detect Java project with build.gradle. Got: {}",
        stderr
    );
}

/// Test Kotlin project detection with .kt files
#[test]
fn test_detect_kotlin_via_kt_files() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    fs::write(
        project.join("build.gradle.kts"),
        r#"
plugins {
    kotlin("jvm") version "1.9.0"
}
group = "com.test"
version = "1.0"
"#,
    )
    .unwrap();
    
    fs::create_dir_all(project.join("src/main/kotlin")).unwrap();
    fs::write(
        project.join("src/main/kotlin/Main.kt"),
        "fun main() { println(\"Hello\") }",
    )
    .unwrap();
    
    // Create a .kt file in root to trigger Kotlin detection
    fs::write(project.join("test.kt"), "// test").unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("main")
        .arg("--depth")
        .arg("1");
    
    let output = cmd.assert().failure();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).unwrap();
    
    assert!(
        stderr.contains("kotlin-language-server") || stderr.contains("Kotlin"),
        "Should detect Kotlin project. Got: {}",
        stderr
    );
}

/// Test that build.gradle.kts without .kt files defaults to Java
#[test]
fn test_gradle_kts_without_kt_files_is_java() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    // build.gradle.kts can be used for Java projects too
    fs::write(
        project.join("build.gradle.kts"),
        r#"
plugins {
    java
}
"#,
    )
    .unwrap();
    
    fs::create_dir_all(project.join("src/main/java")).unwrap();
    fs::write(
        project.join("src/main/java/Main.java"),
        "public class Main { }",
    )
    .unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("Main")
        .arg("--depth")
        .arg("1");
    
    let output = cmd.assert().failure();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).unwrap();
    
    // Without .kt files, should detect as Java
    assert!(
        stderr.contains("jdtls") || stderr.contains("Java"),
        "Should detect as Java without .kt files. Got: {}",
        stderr
    );
}

/// Test error message when no manifest found mentions all languages
#[test]
fn test_no_manifest_error_mentions_all_languages() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    // Empty directory - no manifest files
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(project.join("src/test.txt"), "not a project").unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("Test")
        .arg("--depth")
        .arg("1");
    
    let output = cmd.assert().failure();
    let stderr = String::from_utf8(output.get_output().stderr.clone()).unwrap();
    
    // When no manifest is found, it defaults to Rust and shows warning
    // OR shows error about not detecting language
    assert!(
        stderr.contains("Could not detect") || stderr.contains("defaulting to Rust"),
        "Error should indicate language detection issue. Got: {}",
        stderr
    );
}

/// Test TypeScript detection still works
#[test]
fn test_typescript_detection_unchanged() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    fs::write(
        project.join("package.json"),
        r#"{"name": "test", "version": "1.0.0"}"#,
    )
    .unwrap();
    
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("src/index.ts"),
        "export class Test { }",
    )
    .unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("Test")
        .arg("--depth")
        .arg("1");
    
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should either succeed (if typescript-language-server installed)
    // or fail with TypeScript-related error
    if !output.status.success() {
        assert!(
            stderr.contains("typescript-language-server") || stderr.contains("TypeScript"),
            "Should detect TypeScript project. Got: {}",
            stderr
        );
    } else {
        assert!(
            stdout.contains("TypeScript"),
            "Success output should mention TypeScript. Got: {}",
            stdout
        );
    }
}

/// Test Go detection still works
#[test]
fn test_go_detection_unchanged() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    fs::write(
        project.join("go.mod"),
        "module test\n\ngo 1.21\n",
    )
    .unwrap();
    
    fs::write(
        project.join("main.go"),
        "package main\n\ntype Test struct { }",
    )
    .unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("Test")
        .arg("--depth")
        .arg("1");
    
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Should either succeed (if gopls installed) or fail with Go-related error
    if !output.status.success() {
        assert!(
            stderr.contains("gopls") || stderr.contains("Go"),
            "Should detect Go project. Got: {}",
            stderr
        );
    } else {
        assert!(
            stdout.contains("Go"),
            "Success output should mention Go. Got: {}",
            stdout
        );
    }
}

/// Test Rust detection still works
#[test]
fn test_rust_detection_unchanged() {
    let temp = TempDir::new().unwrap();
    let project = temp.path();
    
    fs::write(
        project.join("Cargo.toml"),
        r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("src/lib.rs"),
        "pub struct Test { }",
    )
    .unwrap();
    
    let mut cmd = Command::cargo_bin("typeglass").unwrap();
    cmd.current_dir(project)
        .arg("from")
        .arg("--symbol")
        .arg("Test")
        .arg("--depth")
        .arg("1");
    
    // This should succeed because rust-analyzer is installed
    // But if it fails, it should mention rust-analyzer
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    
    if !output.status.success() {
        assert!(
            stderr.contains("rust-analyzer") || stderr.contains("Rust"),
            "If Rust detection fails, should mention rust-analyzer. Got: {}",
            stderr
        );
    }
}
