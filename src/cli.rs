use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "dep-why")]
#[command(version)]
#[command(about = "Trace why any dependency exists in your project")]
#[command(long_about = "Find all paths from your direct dependencies to any transitive dependency. Supports npm, cargo, and pip.")]
pub struct Args {
    /// Package name to search for
    #[arg(value_name = "PACKAGE")]
    pub package: String,
    
    /// Show all paths (default: show up to 5 shortest)
    #[arg(long, short = 'a')]
    pub all: bool,
    
    /// Maximum depth to search (default: unlimited)
    #[arg(long, short = 'd')]
    pub depth: Option<usize>,
    
    /// Output format
    #[arg(long, short = 'f', value_enum, default_value = "tree")]
    pub format: OutputFormat,
    
    /// Force ecosystem detection
    #[arg(long, short = 'e', value_enum)]
    pub ecosystem: Option<Ecosystem>,
    
    /// Path to lock file
    #[arg(long, short = 'l')]
    pub lock_file: Option<PathBuf>,
    
    /// Include dev dependencies in search
    #[arg(long)]
    pub include_dev: bool,
    
    /// Only match specific version
    #[arg(long, short = 'v')]
    pub version_match: Option<String>,
    
    /// Minimal output (exit 0 if found, for scripts)
    #[arg(long, short = 'q')]
    pub quiet: bool,
    
    /// Project directory (default: current directory)
    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum Ecosystem {
    Npm,
    Cargo,
    Pip,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    #[default]
    Tree,
    Json,
    Mermaid,
}

pub fn parse_args() -> Args {
    Args::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_only() {
        let args = Args::parse_from(["dep-why", "lodash"]);
        assert_eq!(args.package, "lodash");
        assert!(!args.all);
        assert!(matches!(args.format, OutputFormat::Tree));
    }

    #[test]
    fn test_parse_all_flag() {
        let args = Args::parse_from(["dep-why", "--all", "lodash"]);
        assert!(args.all);
    }

    #[test]
    fn test_parse_all_short() {
        let args = Args::parse_from(["dep-why", "-a", "lodash"]);
        assert!(args.all);
    }

    #[test]
    fn test_parse_depth() {
        let args = Args::parse_from(["dep-why", "-d", "5", "lodash"]);
        assert_eq!(args.depth, Some(5));
    }

    #[test]
    fn test_parse_depth_long() {
        let args = Args::parse_from(["dep-why", "--depth", "10", "lodash"]);
        assert_eq!(args.depth, Some(10));
    }

    #[test]
    fn test_parse_format_json() {
        let args = Args::parse_from(["dep-why", "-f", "json", "lodash"]);
        assert!(matches!(args.format, OutputFormat::Json));
    }

    #[test]
    fn test_parse_format_mermaid() {
        let args = Args::parse_from(["dep-why", "--format", "mermaid", "lodash"]);
        assert!(matches!(args.format, OutputFormat::Mermaid));
    }

    #[test]
    fn test_parse_ecosystem() {
        let args = Args::parse_from(["dep-why", "-e", "npm", "lodash"]);
        assert!(matches!(args.ecosystem, Some(Ecosystem::Npm)));
    }

    #[test]
    fn test_parse_ecosystem_long() {
        let args = Args::parse_from(["dep-why", "--ecosystem", "cargo", "serde"]);
        assert!(matches!(args.ecosystem, Some(Ecosystem::Cargo)));
    }

    #[test]
    fn test_parse_lock_file() {
        let args = Args::parse_from(["dep-why", "-l", "/tmp/lock", "lodash"]);
        assert_eq!(args.lock_file, Some(PathBuf::from("/tmp/lock")));
    }

    #[test]
    fn test_parse_include_dev() {
        let args = Args::parse_from(["dep-why", "--include-dev", "jest"]);
        assert!(args.include_dev);
    }

    #[test]
    fn test_parse_version_match() {
        let args = Args::parse_from(["dep-why", "-v", "4.17.21", "lodash"]);
        assert_eq!(args.version_match, Some("4.17.21".to_string()));
    }

    #[test]
    fn test_parse_quiet() {
        let args = Args::parse_from(["dep-why", "-q", "lodash"]);
        assert!(args.quiet);
    }

    #[test]
    fn test_parse_quiet_long() {
        let args = Args::parse_from(["dep-why", "--quiet", "lodash"]);
        assert!(args.quiet);
    }

    #[test]
    fn test_parse_directory() {
        let args = Args::parse_from(["dep-why", "--dir", "/tmp/project", "lodash"]);
        assert_eq!(args.dir, Some(PathBuf::from("/tmp/project")));
    }

    #[test]
    fn test_default_depth_unlimited() {
        let args = Args::parse_from(["dep-why", "lodash"]);
        assert!(args.depth.is_none()); // unlimited by default per spec
    }
}
