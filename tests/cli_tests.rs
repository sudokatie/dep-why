use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("dep-why").unwrap()
}

fn create_npm_project(dir: &std::path::Path) {
    fs::write(dir.join("package-lock.json"), r#"{
        "name": "test-project",
        "lockfileVersion": 3,
        "packages": {
            "": {
                "name": "test-project",
                "version": "1.0.0",
                "dependencies": {
                    "express": "^4.18.0"
                }
            },
            "node_modules/express": {
                "version": "4.18.2",
                "dependencies": {
                    "accepts": "~1.3.8"
                }
            },
            "node_modules/accepts": {
                "version": "1.3.8",
                "dependencies": {
                    "mime-types": "~2.1.34"
                }
            },
            "node_modules/mime-types": {
                "version": "2.1.35"
            }
        }
    }"#).unwrap();
}

fn create_cargo_project(dir: &std::path::Path) {
    fs::write(dir.join("Cargo.lock"), r#"
[[package]]
name = "myapp"
version = "0.1.0"
dependencies = [
    "tokio 1.0.0",
]

[[package]]
name = "tokio"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
dependencies = [
    "mio 0.8.0",
]

[[package]]
name = "mio"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#).unwrap();
}

#[test]
fn test_help_flag() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("dep-why"));
}

#[test]
fn test_version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("dep-why"));
}

#[test]
fn test_missing_package_arg() {
    cmd()
        .assert()
        .failure();
}

#[test]
fn test_find_direct_dependency() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("express")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("express"));
}

#[test]
fn test_find_transitive_dependency() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("accepts")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("accepts"));
}

#[test]
fn test_find_deep_dependency() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("mime-types")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("mime-types"));
}

#[test]
fn test_package_not_found() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    // Per spec: package not found is exit 0 (it's a valid answer)
    cmd()
        .arg("nonexistent-package")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("not in your dependency tree"));
}

#[test]
fn test_no_lock_file() {
    let dir = TempDir::new().unwrap();
    
    cmd()
        .arg("lodash")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No lock file"));
}

#[test]
fn test_json_output() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("express")
        .arg("--dir")
        .arg(dir.path())
        .arg("-f")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"target\":"))
        .stdout(predicate::str::contains("\"summary\":"));
}

#[test]
fn test_mermaid_output() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("express")
        .arg("--dir")
        .arg(dir.path())
        .arg("-f")
        .arg("mermaid")
        .assert()
        .success()
        .stdout(predicate::str::contains("graph TD"))
        .stdout(predicate::str::contains("ROOT"));
}

#[test]
fn test_depth_flag() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    // With depth 1, should not find deep deps (mime-types is at depth 3)
    cmd()
        .arg("mime-types")
        .arg("--dir")
        .arg(dir.path())
        .arg("-d")
        .arg("1")
        .assert()
        .success()
        .stderr(predicate::str::contains("not reachable"));
}

#[test]
fn test_all_paths_flag() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("accepts")
        .arg("--dir")
        .arg(dir.path())
        .arg("--all")
        .assert()
        .success();
}

#[test]
fn test_ecosystem_flag() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("express")
        .arg("--dir")
        .arg(dir.path())
        .arg("-e")
        .arg("npm")
        .assert()
        .success();
}

#[test]
fn test_ecosystem_cargo() {
    let dir = TempDir::new().unwrap();
    create_cargo_project(dir.path());
    
    cmd()
        .arg("tokio")
        .arg("--dir")
        .arg(dir.path())
        .arg("-e")
        .arg("cargo")
        .assert()
        .success();
}

#[test]
fn test_quiet_mode_found() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    // Quiet mode: exit 0 if found, no output
    cmd()
        .arg("express")
        .arg("--dir")
        .arg(dir.path())
        .arg("-q")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_quiet_mode_not_found() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    // Quiet mode: exit 0 even if not found (per spec)
    cmd()
        .arg("nonexistent")
        .arg("--dir")
        .arg(dir.path())
        .arg("-q")
        .assert()
        .success();
}

#[test]
fn test_lock_file_flag() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("express")
        .arg("-l")
        .arg(dir.path().join("package-lock.json"))
        .assert()
        .success();
}

#[test]
fn test_invalid_directory() {
    cmd()
        .arg("lodash")
        .arg("--dir")
        .arg("/nonexistent/path/that/does/not/exist")
        .assert()
        .failure();
}

#[test]
fn test_tree_output_has_summary() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("accepts")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Summary:"))
        .stdout(predicate::str::contains("Direct dependents:"));
}

#[test]
fn test_tree_output_has_found_via() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("accepts")
        .arg("--dir")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Found via:"));
}
