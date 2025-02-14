mod common;

use std::process::Command;

#[test]
fn test_ns() {
    // Test case 1: Check if the command executes successfully
    let status = Command::new("cargo")
        .args(&["run", "--bin", "ns", "echo hello"])
        .status()
        .expect("Failed to execute command");
    assert!(status.success());

    // Test case 2: Check if multiple commands execute sequentially
    let status = Command::new("cargo")
        .args(&["run", "--bin", "ns", "echo hello", "echo world"])
        .status()
        .expect("Failed to execute command");
    assert!(status.success());

    // Test case 3: Check if a command fails
    let status = Command::new("cargo")
        .args(&["run", "--bin", "ns", "false"])
        .status()
        .expect("Failed to execute command");
    assert!(!status.success());

    // Test case 4: Check if one command fails and another works
    let status = Command::new("cargo")
        .args(&["run", "--bin", "ns", "false", "echo hello"])
        .status()
        .expect("Failed to execute command");
    assert!(!status.success());

    // Test case 5: Check if a command with arguments works
    let status = Command::new("cargo")
        .args(&["run", "--bin", "ns", "echo hello world"])
        .status()
        .expect("Failed to execute command");
    assert!(status.success());
}
