use std::process::ExitCode;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("package not found: {0}")]
    PackageNotFound(String),
    
    #[error("no lock file found in {0}")]
    NoLockFile(String),
    
    #[error("unsupported package manager")]
    UnsupportedManager,
    
    #[error("failed to parse lock file: {0}")]
    ParseError(String),
    
    #[error("invalid path: {0}")]
    InvalidPath(String),
    
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("max depth exceeded")]
    MaxDepthExceeded,
}

impl Error {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Error::PackageNotFound(_) => ExitCode::from(1),
            Error::NoLockFile(_) => ExitCode::from(2),
            Error::UnsupportedManager => ExitCode::from(2),
            Error::ParseError(_) => ExitCode::from(3),
            Error::InvalidPath(_) => ExitCode::from(2),
            Error::Io(_) => ExitCode::from(4),
            Error::Json(_) | Error::Toml(_) => ExitCode::from(3),
            Error::MaxDepthExceeded => ExitCode::from(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_not_found_message() {
        let err = Error::PackageNotFound("lodash".to_string());
        assert!(err.to_string().contains("lodash"));
    }

    #[test]
    fn test_no_lock_file_message() {
        let err = Error::NoLockFile("/tmp/project".to_string());
        assert!(err.to_string().contains("/tmp/project"));
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(Error::PackageNotFound("x".into()).exit_code(), ExitCode::from(1));
        assert_eq!(Error::NoLockFile("x".into()).exit_code(), ExitCode::from(2));
        assert_eq!(Error::UnsupportedManager.exit_code(), ExitCode::from(2));
        assert_eq!(Error::ParseError("x".into()).exit_code(), ExitCode::from(3));
        assert_eq!(Error::MaxDepthExceeded.exit_code(), ExitCode::from(5));
    }
}
