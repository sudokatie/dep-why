pub mod cli;
pub mod config;
pub mod error;
pub mod graph;
pub mod output;
pub mod parsers;

use cli::{Args, OutputFormat, PackageManager};
pub use error::{Error, Result};
use graph::{DependencyPath, PathFinder};
use output::{TreeOutput, JsonOutput, MermaidOutput, OutputFormat as OutputTrait};
use parsers::{detect_manager, Parser, NpmParser, CargoParser, PipParser};
use std::path::PathBuf;

/// Run the dependency tracer with the given arguments
pub fn run(args: Args) -> Result<()> {
    // Determine working directory
    let dir = args.dir.clone().unwrap_or_else(|| PathBuf::from("."));
    
    if !dir.exists() {
        return Err(Error::InvalidPath(format!("directory not found: {}", dir.display())));
    }
    
    // Detect or use specified package manager
    let lock_file = if let Some(ref manager) = args.manager {
        // Use specified manager, find appropriate lock file
        let lock_path = match manager {
            PackageManager::Npm => dir.join("package-lock.json"),
            PackageManager::Cargo => dir.join("Cargo.lock"),
            PackageManager::Pip => {
                let pipfile = dir.join("Pipfile.lock");
                if pipfile.exists() {
                    pipfile
                } else {
                    dir.join("poetry.lock")
                }
            }
        };
        
        if !lock_path.exists() {
            return Err(Error::NoLockFile(format!("{} not found", lock_path.display())));
        }
        
        parsers::LockFile {
            path: lock_path,
            manager: manager.clone(),
        }
    } else {
        detect_manager(&dir).ok_or_else(|| {
            Error::NoLockFile(format!("no lock file found in {}", dir.display()))
        })?
    };
    
    // Parse lock file
    let graph = match lock_file.manager {
        PackageManager::Npm => NpmParser.parse(&lock_file.path)?,
        PackageManager::Cargo => CargoParser.parse(&lock_file.path)?,
        PackageManager::Pip => PipParser.parse(&lock_file.path)?,
    };
    
    // Find target package
    let target = graph.get_package(&args.package);
    if target.is_none() {
        return Err(Error::PackageNotFound(args.package.clone()));
    }
    let target_idx = target.unwrap();
    
    // Search for paths
    let finder = PathFinder::new(&graph, args.max_depth);
    let paths: Vec<DependencyPath> = if args.all {
        finder.find_all(target_idx)
    } else {
        finder.find_shortest(target_idx).into_iter().collect()
    };
    
    if paths.is_empty() {
        // Package exists but is not reachable from roots
        eprintln!("Package '{}' exists but is not reachable from direct dependencies.", args.package);
        eprintln!("It may be an orphaned or optional dependency.");
        return Ok(());
    }
    
    // Format output
    let output = match args.format {
        OutputFormat::Tree => TreeOutput.format(&graph, &paths, args.versions)?,
        OutputFormat::Json => JsonOutput.format(&graph, &paths, args.versions)?,
        OutputFormat::Mermaid => MermaidOutput.format(&graph, &paths, args.versions)?,
    };
    
    print!("{}", output);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

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
                    "version": "1.3.8"
                }
            }
        }"#).unwrap();
    }

    #[test]
    fn test_run_finds_package() {
        let dir = TempDir::new().unwrap();
        create_npm_project(dir.path());
        
        let args = Args {
            package: "accepts".to_string(),
            all: false,
            dir: Some(dir.path().to_path_buf()),
            manager: None,
            format: OutputFormat::Tree,
            max_depth: 20,
            versions: false,
            config: None,
        };
        
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_package_not_found() {
        let dir = TempDir::new().unwrap();
        create_npm_project(dir.path());
        
        let args = Args {
            package: "nonexistent".to_string(),
            all: false,
            dir: Some(dir.path().to_path_buf()),
            manager: None,
            format: OutputFormat::Tree,
            max_depth: 20,
            versions: false,
            config: None,
        };
        
        let result = run(args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PackageNotFound(_)));
    }

    #[test]
    fn test_run_no_lock_file() {
        let dir = TempDir::new().unwrap();
        
        let args = Args {
            package: "lodash".to_string(),
            all: false,
            dir: Some(dir.path().to_path_buf()),
            manager: None,
            format: OutputFormat::Tree,
            max_depth: 20,
            versions: false,
            config: None,
        };
        
        let result = run(args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NoLockFile(_)));
    }

    #[test]
    fn test_run_json_format() {
        let dir = TempDir::new().unwrap();
        create_npm_project(dir.path());
        
        let args = Args {
            package: "accepts".to_string(),
            all: false,
            dir: Some(dir.path().to_path_buf()),
            manager: None,
            format: OutputFormat::Json,
            max_depth: 20,
            versions: false,
            config: None,
        };
        
        let result = run(args);
        assert!(result.is_ok());
    }
}
