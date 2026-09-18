//! Integration tests for ProofChain CLI
//! 
//! These tests verify the end-to-end functionality of the CLI commands.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;
use std::fs;
use std::io::Write;

fn create_temp_file(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    temp_file
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: proofchain"));
}

#[test]
fn test_init_command() {
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "init"]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Database initialized successfully"))
        .stdout(predicate::str::contains("Genesis block created"));
}

#[test]
fn test_add_and_verify_workflow() {
    // Create a temporary database and file
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    let content = "Test content for proof-of-existence";
    let test_file = create_temp_file(content);
    let test_path = test_file.path().to_str().unwrap();
    
    // Initialize database
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "init"]);
    cmd.assert().success();
    
    // Add file to ledger
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "add", test_path]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("File registered successfully"))
        .stdout(predicate::str::contains("Hash:"));
    
    // Verify file (should succeed with same content)
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "verify", test_path]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("File verified"));
}

#[test]
fn test_verify_modified_file_fails() {
    // Create a temporary database and file
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    let original_content = "Original content";
    let modified_content = "Modified content - different!";
    
    let test_file = create_temp_file(original_content);
    let test_path = test_file.path().to_str().unwrap();
    
    // Initialize database
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "init"]);
    cmd.assert().success();
    
    // Add original file to ledger
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "add", test_path]);
    cmd.assert().success();
    
    // Modify the file content
    fs::write(test_path, modified_content).unwrap();
    
    // Verify should fail (file not found with new hash)
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "verify", test_path]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("File not found in registry"));
}

#[test]
fn test_check_command_valid_chain() {
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    // Initialize database
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "init"]);
    cmd.assert().success();
    
    // Add a file
    let test_file = create_temp_file("Test content");
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "add", test_file.path().to_str().unwrap()]);
    cmd.assert().success();
    
    // Check chain integrity
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "check"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Integrity: ✓ VALID"));
}

#[test]
fn test_list_command() {
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    // Initialize database
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "init"]);
    cmd.assert().success();
    
    // Add a file
    let test_file = create_temp_file("List test content");
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "add", test_file.path().to_str().unwrap()]);
    cmd.assert().success();
    
    // List files
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "list"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Registered Files"))
        .stdout(predicate::str::contains("Height"));
}

#[test]
fn test_status_command() {
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    // Initialize database
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "init"]);
    cmd.assert().success();
    
    // Check status
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "status"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ProofChain Status"))
        .stdout(predicate::str::contains("Block Height: 1"))
        .stdout(predicate::str::contains("Chain Valid: true"));
}

#[test]
fn test_duplicate_file_registration() {
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    let content = "Duplicate test content";
    let test_file = create_temp_file(content);
    let test_path = test_file.path().to_str().unwrap();
    
    // Initialize database
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "init"]);
    cmd.assert().success();
    
    // Add file first time
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "add", test_path]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("File registered successfully"));
    
    // Add same file again (should detect duplicate)
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "add", test_path]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("File already registered"));
}

#[test]
fn test_nonexistent_file_error() {
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    // Initialize database
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "init"]);
    cmd.assert().success();
    
    // Try to add non-existent file
    let mut cmd = Command::cargo_bin("proofchain").unwrap();
    cmd.args(&["--db", db_path, "add", "/nonexistent/path/file.txt"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));
}
