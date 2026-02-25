use crate::cli::Ecosystem;
use std::path::{Path, PathBuf};

/// Information about a detected lock file
#[derive(Debug, Clone)]
pub struct LockFile {
    pub path: PathBuf,
    pub ecosystem: Ecosystem,
}

/// Detect which package manager is used in a directory
/// Priority order: npm > cargo > pip (per spec Section 4.2)
pub fn detect_ecosystem(dir: &Path) -> Option<LockFile> {
    // Check for npm
    let npm_lock = dir.join("package-lock.json");
    if npm_lock.exists() {
        return Some(LockFile {
            path: npm_lock,
            ecosystem: Ecosystem::Npm,
        });
    }
    
    // Check for Cargo
    let cargo_lock = dir.join("Cargo.lock");
    if cargo_lock.exists() {
        return Some(LockFile {
            path: cargo_lock,
            ecosystem: Ecosystem::Cargo,
        });
    }
    
    // Check for pip (Pipfile.lock has priority)
    let pipfile_lock = dir.join("Pipfile.lock");
    if pipfile_lock.exists() {
        return Some(LockFile {
            path: pipfile_lock,
            ecosystem: Ecosystem::Pip,
        });
    }
    
    let poetry_lock = dir.join("poetry.lock");
    if poetry_lock.exists() {
        return Some(LockFile {
            path: poetry_lock,
            ecosystem: Ecosystem::Pip,
        });
    }
    
    None
}

/// Detect ecosystem from a lock file path
pub fn detect_from_path(path: &Path) -> Option<Ecosystem> {
    let filename = path.file_name()?.to_str()?;
    
    match filename {
        "package-lock.json" => Some(Ecosystem::Npm),
        "Cargo.lock" => Some(Ecosystem::Cargo),
        "Pipfile.lock" | "poetry.lock" => Some(Ecosystem::Pip),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;

    #[test]
    fn test_detect_npm() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("package-lock.json")).unwrap();
        
        let result = detect_ecosystem(dir.path());
        assert!(result.is_some());
        assert!(matches!(result.unwrap().ecosystem, Ecosystem::Npm));
    }

    #[test]
    fn test_detect_cargo() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("Cargo.lock")).unwrap();
        
        let result = detect_ecosystem(dir.path());
        assert!(result.is_some());
        assert!(matches!(result.unwrap().ecosystem, Ecosystem::Cargo));
    }

    #[test]
    fn test_detect_pipfile() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("Pipfile.lock")).unwrap();
        
        let result = detect_ecosystem(dir.path());
        assert!(result.is_some());
        assert!(matches!(result.unwrap().ecosystem, Ecosystem::Pip));
    }

    #[test]
    fn test_detect_poetry() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("poetry.lock")).unwrap();
        
        let result = detect_ecosystem(dir.path());
        assert!(result.is_some());
        assert!(matches!(result.unwrap().ecosystem, Ecosystem::Pip));
    }

    #[test]
    fn test_detect_none() {
        let dir = TempDir::new().unwrap();
        let result = detect_ecosystem(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_npm_priority_over_cargo() {
        // If both exist, npm should be detected first
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("package-lock.json")).unwrap();
        File::create(dir.path().join("Cargo.lock")).unwrap();
        
        let result = detect_ecosystem(dir.path());
        assert!(matches!(result.unwrap().ecosystem, Ecosystem::Npm));
    }

    #[test]
    fn test_detect_from_path() {
        assert!(matches!(
            detect_from_path(Path::new("/tmp/package-lock.json")),
            Some(Ecosystem::Npm)
        ));
        assert!(matches!(
            detect_from_path(Path::new("/tmp/Cargo.lock")),
            Some(Ecosystem::Cargo)
        ));
        assert!(matches!(
            detect_from_path(Path::new("/tmp/Pipfile.lock")),
            Some(Ecosystem::Pip)
        ));
        assert!(matches!(
            detect_from_path(Path::new("/tmp/poetry.lock")),
            Some(Ecosystem::Pip)
        ));
        assert!(detect_from_path(Path::new("/tmp/unknown.txt")).is_none());
    }
}
