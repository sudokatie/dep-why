use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "dep-why")]
#[command(version)]
#[command(about = "Trace why any dependency exists in your project")]
#[command(long_about = "Find all paths from your direct dependencies to any transitive dependency. Supports npm, cargo, and pip.")]
pub struct Args {
    /// Package name to trace
    #[arg(value_name = "PACKAGE")]
    pub package: String,
    
    /// Show all paths (default: only shortest)
    #[arg(long, short = 'a')]
    pub all: bool,
    
    /// Project directory (default: current directory)
    #[arg(long, short = 'd', value_name = "DIR")]
    pub dir: Option<PathBuf>,
    
    /// Force specific package manager
    #[arg(long, value_enum)]
    pub manager: Option<PackageManager>,
    
    /// Output format
    #[arg(long, short = 'f', value_enum, default_value = "tree")]
    pub format: OutputFormat,
    
    /// Maximum depth to search
    #[arg(long, default_value = "20")]
    pub max_depth: usize,
    
    /// Show dependency versions in output
    #[arg(long, short = 'v')]
    pub versions: bool,
    
    /// Path to config file
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum PackageManager {
    Npm,
    Cargo,
    Pip,
}

#[derive(Clone, Debug, Default, ValueEnum)]
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
    fn test_parse_directory() {
        let args = Args::parse_from(["dep-why", "-d", "/tmp/project", "lodash"]);
        assert_eq!(args.dir, Some(PathBuf::from("/tmp/project")));
    }

    #[test]
    fn test_parse_manager() {
        let args = Args::parse_from(["dep-why", "--manager", "npm", "lodash"]);
        assert!(matches!(args.manager, Some(PackageManager::Npm)));
    }

    #[test]
    fn test_parse_max_depth() {
        let args = Args::parse_from(["dep-why", "--max-depth", "10", "lodash"]);
        assert_eq!(args.max_depth, 10);
    }

    #[test]
    fn test_parse_versions() {
        let args = Args::parse_from(["dep-why", "-v", "lodash"]);
        assert!(args.versions);
    }

    #[test]
    fn test_default_max_depth() {
        let args = Args::parse_from(["dep-why", "lodash"]);
        assert_eq!(args.max_depth, 20);
    }
}
