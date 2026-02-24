use crate::cli::PackageManager;
use std::path::{Path, PathBuf};

/// Information about a detected lock file
#[derive(Debug, Clone)]
pub struct LockFile {
    pub path: PathBuf,
    pub manager: PackageManager,
}

/// Detect which package manager is used in a directory
pub fn detect_manager(dir: &Path) -> Option<LockFile> {
    // Check for npm
    let npm_lock = dir.join("package-lock.json");
    if npm_lock.exists() {
        return Some(LockFile {
            path: npm_lock,
            manager: PackageManager::Npm,
        });
    }
    
    // Check for Cargo
    let cargo_lock = dir.join("Cargo.lock");
    if cargo_lock.exists() {
        return Some(LockFile {
            path: cargo_lock,
            manager: PackageManager::Cargo,
        });
    }
    
    // Check for pip (Pipfile.lock has priority)
    let pipfile_lock = dir.join("Pipfile.lock");
    if pipfile_lock.exists() {
        return Some(LockFile {
            path: pipfile_lock,
            manager: PackageManager::Pip,
        });
    }
    
    let poetry_lock = dir.join("poetry.lock");
    if poetry_lock.exists() {
        return Some(LockFile {
            path: poetry_lock,
            manager: PackageManager::Pip,
        });
    }
    
    None
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
        
        let result = detect_manager(dir.path());
        assert!(result.is_some());
        assert!(matches!(result.unwrap().manager, PackageManager::Npm));
    }

    #[test]
    fn test_detect_cargo() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("Cargo.lock")).unwrap();
        
        let result = detect_manager(dir.path());
        assert!(result.is_some());
        assert!(matches!(result.unwrap().manager, PackageManager::Cargo));
    }

    #[test]
    fn test_detect_pipfile() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("Pipfile.lock")).unwrap();
        
        let result = detect_manager(dir.path());
        assert!(result.is_some());
        assert!(matches!(result.unwrap().manager, PackageManager::Pip));
    }

    #[test]
    fn test_detect_poetry() {
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("poetry.lock")).unwrap();
        
        let result = detect_manager(dir.path());
        assert!(result.is_some());
        assert!(matches!(result.unwrap().manager, PackageManager::Pip));
    }

    #[test]
    fn test_detect_none() {
        let dir = TempDir::new().unwrap();
        let result = detect_manager(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_npm_priority_over_cargo() {
        // If both exist, npm should be detected first
        let dir = TempDir::new().unwrap();
        File::create(dir.path().join("package-lock.json")).unwrap();
        File::create(dir.path().join("Cargo.lock")).unwrap();
        
        let result = detect_manager(dir.path());
        assert!(matches!(result.unwrap().manager, PackageManager::Npm));
    }
}
