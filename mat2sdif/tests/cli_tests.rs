//! Integration tests for mat2sdif CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get the mat2sdif command.
fn mat2sdif() -> Command {
    Command::cargo_bin("mat2sdif").unwrap()
}

// ============================================================================
// Basic CLI Tests
// ============================================================================

#[test]
fn test_help() {
    mat2sdif()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Convert MATLAB/Octave"))
        .stdout(predicate::str::contains("--list"))
        .stdout(predicate::str::contains("--time-var"))
        .stdout(predicate::str::contains("EXAMPLES"));
}

#[test]
fn test_version() {
    mat2sdif()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("mat2sdif"));
}

#[test]
fn test_missing_input() {
    mat2sdif()
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_nonexistent_input() {
    mat2sdif()
        .arg("/nonexistent/file.mat")
        .arg("output.sdif")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_invalid_frame_type() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("test.mat");

    // Create a dummy file (will fail to parse, but that's after arg validation)
    fs::write(&input, "dummy").unwrap();

    mat2sdif()
        .arg(&input)
        .arg("output.sdif")
        .arg("--frame-type")
        .arg("TOOLONG")
        .assert()
        .failure()
        .stderr(predicate::str::contains("4 characters"));
}

// ============================================================================
// List Mode Tests
// ============================================================================

#[test]
fn test_list_missing_file() {
    mat2sdif()
        .arg("--list")
        .arg("/nonexistent/file.mat")
        .assert()
        .failure();
}

// ============================================================================
// Dry Run Tests
// ============================================================================

#[test]
fn test_dry_run_missing_file() {
    mat2sdif()
        .arg("--dry-run")
        .arg("/nonexistent/file.mat")
        .arg("output.sdif")
        .assert()
        .failure();
}

// ============================================================================
// Tests requiring fixture files (marked ignore)
// ============================================================================

#[test]
#[ignore = "Requires test fixture: simple.mat"]
fn test_list_simple_mat() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple.mat");

    if !fixture.exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }

    mat2sdif()
        .arg("--list")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("Variables in"));
}

#[test]
#[ignore = "Requires test fixture: simple.mat"]
fn test_convert_simple_mat() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple.mat");

    if !fixture.exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }

    let temp = TempDir::new().unwrap();
    let output = temp.path().join("output.sdif");

    mat2sdif()
        .arg(&fixture)
        .arg(&output)
        .arg("-v")
        .assert()
        .success()
        .stdout(predicate::str::contains("success"));

    assert!(output.exists());
}

#[test]
#[ignore = "Requires test fixture: simple.mat"]
fn test_dry_run_simple_mat() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple.mat");

    if !fixture.exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }

    let temp = TempDir::new().unwrap();
    let output = temp.path().join("output.sdif");

    mat2sdif()
        .arg("--dry-run")
        .arg(&fixture)
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"))
        .stdout(predicate::str::contains("Validation passed"));

    // Output should NOT be created in dry-run mode
    assert!(!output.exists());
}
