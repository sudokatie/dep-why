use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{package} is not in your dependency tree")]
    PackageNotFound { package: String },
    
    #[error("{package}@{version} not found (available: {available})")]
    VersionNotFound {
        package: String,
        version: String,
        available: String,
    },
    
    #[error("No lock file found. Run npm install, cargo build, or pip install first.")]
    NoLockFile,
    
    #[error("Failed to parse {path}: {message}")]
    ParseError { path: PathBuf, message: String },
    
    #[error("Lock file format not supported: {0}")]
    UnsupportedFormat(PathBuf),
    
    #[error("Cannot read {path}: {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError {
            path: PathBuf::from("<unknown>"),
            source: e,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerializationError(e.to_string())
    }
}

impl Error {
    /// Exit codes per spec Section 7.3
    /// 0: Success (including "not found" - that's a valid answer)
    /// 1: Error (couldn't complete the query)
    pub fn exit_code(&self) -> ExitCode {
        match self {
            // Package not found is exit 0 - it's a valid answer to the query
            Error::PackageNotFound { .. } => ExitCode::SUCCESS,
            Error::VersionNotFound { .. } => ExitCode::SUCCESS,
            // Actual errors are exit 1
            Error::NoLockFile => ExitCode::FAILURE,
            Error::ParseError { .. } => ExitCode::FAILURE,
            Error::UnsupportedFormat(_) => ExitCode::FAILURE,
            Error::IoError { .. } => ExitCode::FAILURE,
            Error::ConfigError(_) => ExitCode::FAILURE,
            Error::SerializationError(_) => ExitCode::FAILURE,
        }
    }
    
    pub fn parse_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::ParseError {
            path: path.into(),
            message: message.into(),
        }
    }
    
    pub fn io_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::IoError {
            path: path.into(),
            source,
        }
    }
    
    pub fn package_not_found(package: impl Into<String>) -> Self {
        Self::PackageNotFound {
            package: package.into(),
        }
    }
    
    pub fn version_not_found(
        package: impl Into<String>,
        version: impl Into<String>,
        available: impl Into<String>,
    ) -> Self {
        Self::VersionNotFound {
            package: package.into(),
            version: version.into(),
            available: available.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_lock_file_message() {
        let err = Error::NoLockFile;
        let msg = err.to_string();
        assert!(msg.contains("npm install"));
        assert!(msg.contains("cargo build"));
    }

    #[test]
    fn test_parse_error_includes_path() {
        let err = Error::parse_error("/some/path", "invalid JSON");
        let msg = err.to_string();
        assert!(msg.contains("/some/path"));
        assert!(msg.contains("invalid JSON"));
    }

    #[test]
    fn test_package_not_found() {
        let err = Error::package_not_found("lodash");
        let msg = err.to_string();
        assert!(msg.contains("lodash"));
        assert!(msg.contains("not in your dependency tree"));
    }

    #[test]
    fn test_version_not_found_shows_available() {
        let err = Error::version_not_found("lodash", "5.0.0", "4.17.21, 4.17.20");
        let msg = err.to_string();
        assert!(msg.contains("4.17.21"));
        assert!(msg.contains("4.17.20"));
    }

    #[test]
    fn test_package_not_found_exit_code() {
        // Per spec: "not found" is a valid answer, exit 0
        let err = Error::package_not_found("lodash");
        assert_eq!(err.exit_code(), ExitCode::SUCCESS);
    }

    #[test]
    fn test_no_lock_file_exit_code() {
        let err = Error::NoLockFile;
        assert_eq!(err.exit_code(), ExitCode::FAILURE);
    }

    #[test]
    fn test_parse_error_exit_code() {
        let err = Error::parse_error("/tmp/file", "bad");
        assert_eq!(err.exit_code(), ExitCode::FAILURE);
    }
}
