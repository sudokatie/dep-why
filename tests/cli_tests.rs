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
        .arg("-d")
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
        .arg("-d")
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
        .arg("-d")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("mime-types"));
}

#[test]
fn test_package_not_found() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("nonexistent-package")
        .arg("-d")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_no_lock_file() {
    let dir = TempDir::new().unwrap();
    
    cmd()
        .arg("lodash")
        .arg("-d")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no lock file"));
}

#[test]
fn test_json_output() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("express")
        .arg("-d")
        .arg(dir.path())
        .arg("-f")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_paths\""));
}

#[test]
fn test_mermaid_output() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("express")
        .arg("-d")
        .arg(dir.path())
        .arg("-f")
        .arg("mermaid")
        .assert()
        .success()
        .stdout(predicate::str::contains("graph TD"));
}

#[test]
fn test_versions_flag() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("express")
        .arg("-d")
        .arg(dir.path())
        .arg("-v")
        .assert()
        .success()
        .stdout(predicate::str::contains("4.18.2"));
}

#[test]
fn test_all_paths_flag() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("accepts")
        .arg("-d")
        .arg(dir.path())
        .arg("--all")
        .assert()
        .success();
}

#[test]
fn test_force_npm_manager() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    cmd()
        .arg("express")
        .arg("-d")
        .arg(dir.path())
        .arg("--manager")
        .arg("npm")
        .assert()
        .success();
}

#[test]
fn test_force_cargo_manager() {
    let dir = TempDir::new().unwrap();
    create_cargo_project(dir.path());
    
    cmd()
        .arg("tokio")
        .arg("-d")
        .arg(dir.path())
        .arg("--manager")
        .arg("cargo")
        .assert()
        .success();
}

#[test]
fn test_max_depth_limit() {
    let dir = TempDir::new().unwrap();
    create_npm_project(dir.path());
    
    // With max-depth 1, should not find deep deps (mime-types is at depth 3)
    cmd()
        .arg("mime-types")
        .arg("-d")
        .arg(dir.path())
        .arg("--max-depth")
        .arg("1")
        .assert()
        .success()
        .stderr(predicate::str::contains("not reachable"));
}

#[test]
fn test_invalid_directory() {
    cmd()
        .arg("lodash")
        .arg("-d")
        .arg("/nonexistent/path/that/does/not/exist")
        .assert()
        .failure();
}
