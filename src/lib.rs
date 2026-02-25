pub mod cli;
pub mod config;
pub mod error;
pub mod graph;
pub mod output;
pub mod parsers;

use cli::{Args, Ecosystem, OutputFormat};
use config::Config;
pub use error::{Error, Result};
use graph::{PathFinder, SearchOptions};
use output::{TreeOutput, JsonOutput, MermaidOutput, OutputFormat as OutputTrait};
use parsers::{detect_ecosystem, detect_from_path, parse_lock_file};
use std::path::PathBuf;

/// Run the dependency tracer with the given arguments
pub fn run(args: Args) -> Result<()> {
    // Load config
    let config = Config::load(args.lock_file.as_ref()).unwrap_or_default();
    
    // Determine working directory
    let dir = args.dir.clone().unwrap_or_else(|| PathBuf::from("."));
    
    if !dir.exists() {
        return Err(Error::io_error(&dir, std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "directory not found"
        )));
    }
    
    // Determine lock file and ecosystem
    let (lock_path, ecosystem) = if let Some(ref lock_file) = args.lock_file {
        // Explicit lock file path
        if !lock_file.exists() {
            return Err(Error::io_error(lock_file, std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "lock file not found"
            )));
        }
        let eco = args.ecosystem.or_else(|| detect_from_path(lock_file))
            .ok_or(Error::UnsupportedFormat(lock_file.clone()))?;
        (lock_file.clone(), eco)
    } else if let Some(eco) = args.ecosystem {
        // Ecosystem specified, find appropriate lock file
        let lock_path = match eco {
            Ecosystem::Npm => dir.join("package-lock.json"),
            Ecosystem::Cargo => dir.join("Cargo.lock"),
            Ecosystem::Pip => {
                let pipfile = dir.join("Pipfile.lock");
                if pipfile.exists() {
                    pipfile
                } else {
                    dir.join("poetry.lock")
                }
            }
        };
        
        if !lock_path.exists() {
            return Err(Error::NoLockFile);
        }
        
        (lock_path, eco)
    } else {
        // Auto-detect
        let detected = detect_ecosystem(&dir).ok_or(Error::NoLockFile)?;
        (detected.path, detected.ecosystem)
    };
    
    // Parse lock file
    let graph = parse_lock_file(&lock_path, ecosystem)?;
    
    // Find target package
    let target_idx = if let Some(ref version) = args.version_match {
        // Match specific version
        graph.get_package_version(&args.package, version).ok_or_else(|| {
            let versions = graph.get_package_versions(&args.package);
            if versions.is_empty() {
                Error::package_not_found(&args.package)
            } else {
                let available: Vec<String> = versions.iter()
                    .map(|(_, p)| p.version.clone())
                    .collect();
                Error::version_not_found(&args.package, version, available.join(", "))
            }
        })?
    } else {
        graph.get_package(&args.package).ok_or_else(|| {
            Error::package_not_found(&args.package)
        })?
    };
    
    // Configure search options
    // Per spec: depth default is unlimited (use large value internally)
    let search_options = SearchOptions {
        max_depth: args.depth.unwrap_or(usize::MAX),
        max_paths: if args.all { 0 } else { config.max_paths() },
        include_dev: args.include_dev || config.include_dev,
    };
    
    let finder = PathFinder::with_options(&graph, search_options);
    
    // Check if reachable first (fast path for quiet mode)
    if args.quiet {
        if finder.is_reachable(target_idx) {
            // Exit 0 silently - package found
            return Ok(());
        } else {
            // Package exists but not reachable
            eprintln!("{} is not in your dependency tree", args.package);
            return Ok(());
        }
    }
    
    // Build query result
    let result = finder.query(target_idx);
    
    if result.paths.is_empty() {
        // Package exists but is not reachable from roots
        eprintln!("Package '{}' exists but is not reachable from direct dependencies.", args.package);
        eprintln!("It may be an orphaned or optional dependency.");
        if !args.include_dev {
            eprintln!("Try --include-dev to include dev dependencies in the search.");
        }
        return Ok(());
    }
    
    // Determine output format (CLI > config > default)
    let format = args.format;
    
    // Format and print output
    let output = match format {
        OutputFormat::Tree => TreeOutput.format(&graph, &result)?,
        OutputFormat::Json => JsonOutput.format(&graph, &result)?,
        OutputFormat::Mermaid => MermaidOutput.format(&graph, &result)?,
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
            depth: None,
            format: OutputFormat::Tree,
            ecosystem: None,
            lock_file: None,
            include_dev: false,
            version_match: None,
            quiet: false,
            dir: Some(dir.path().to_path_buf()),
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
            depth: None,
            format: OutputFormat::Tree,
            ecosystem: None,
            lock_file: None,
            include_dev: false,
            version_match: None,
            quiet: false,
            dir: Some(dir.path().to_path_buf()),
        };
        
        let result = run(args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PackageNotFound { .. }));
    }

    #[test]
    fn test_run_no_lock_file() {
        let dir = TempDir::new().unwrap();
        
        let args = Args {
            package: "lodash".to_string(),
            all: false,
            depth: None,
            format: OutputFormat::Tree,
            ecosystem: None,
            lock_file: None,
            include_dev: false,
            version_match: None,
            quiet: false,
            dir: Some(dir.path().to_path_buf()),
        };
        
        let result = run(args);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NoLockFile));
    }

    #[test]
    fn test_run_json_format() {
        let dir = TempDir::new().unwrap();
        create_npm_project(dir.path());
        
        let args = Args {
            package: "accepts".to_string(),
            all: false,
            depth: None,
            format: OutputFormat::Json,
            ecosystem: None,
            lock_file: None,
            include_dev: false,
            version_match: None,
            quiet: false,
            dir: Some(dir.path().to_path_buf()),
        };
        
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_quiet_mode() {
        let dir = TempDir::new().unwrap();
        create_npm_project(dir.path());
        
        let args = Args {
            package: "accepts".to_string(),
            all: false,
            depth: None,
            format: OutputFormat::Tree,
            ecosystem: None,
            lock_file: None,
            include_dev: false,
            version_match: None,
            quiet: true,
            dir: Some(dir.path().to_path_buf()),
        };
        
        let result = run(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_with_ecosystem() {
        let dir = TempDir::new().unwrap();
        create_npm_project(dir.path());
        
        let args = Args {
            package: "accepts".to_string(),
            all: false,
            depth: None,
            format: OutputFormat::Tree,
            ecosystem: Some(Ecosystem::Npm),
            lock_file: None,
            include_dev: false,
            version_match: None,
            quiet: false,
            dir: Some(dir.path().to_path_buf()),
        };
        
        let result = run(args);
        assert!(result.is_ok());
    }
}
