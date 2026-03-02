//! Security-related types.

use serde::{Deserialize, Serialize};

/// Severity level for vulnerabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    /// Parse severity from CVSS score.
    pub fn from_cvss_score(score: f64) -> Self {
        if score >= 9.0 {
            Severity::Critical
        } else if score >= 7.0 {
            Severity::High
        } else if score >= 4.0 {
            Severity::Medium
        } else {
            Severity::Low
        }
    }

    /// Parse severity from string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "LOW" => Some(Severity::Low),
            "MEDIUM" | "MODERATE" => Some(Severity::Medium),
            "HIGH" => Some(Severity::High),
            "CRITICAL" => Some(Severity::Critical),
            _ => None,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A single vulnerability.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vulnerability {
    /// Vulnerability ID (e.g., GHSA-xxxx-xxxx-xxxx, CVE-2021-xxxx)
    pub id: String,
    /// Severity level
    pub severity: Severity,
    /// CVSS score (0.0 - 10.0)
    pub score: Option<f64>,
    /// Brief summary
    pub summary: String,
    /// URL to advisory
    pub url: Option<String>,
}

/// Vulnerability info for a package.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VulnerabilityInfo {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Ecosystem (npm, cargo, pip)
    pub ecosystem: String,
    /// List of vulnerabilities
    pub vulnerabilities: Vec<Vulnerability>,
}

impl VulnerabilityInfo {
    /// Check if package has any vulnerabilities.
    pub fn is_vulnerable(&self) -> bool {
        !self.vulnerabilities.is_empty()
    }

    /// Get highest severity.
    pub fn max_severity(&self) -> Option<Severity> {
        self.vulnerabilities.iter().map(|v| v.severity).max()
    }

    /// Filter vulnerabilities by minimum severity.
    pub fn filter_by_severity(&self, min_severity: Severity) -> Vec<&Vulnerability> {
        self.vulnerabilities
            .iter()
            .filter(|v| v.severity >= min_severity)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_from_cvss() {
        assert_eq!(Severity::from_cvss_score(9.5), Severity::Critical);
        assert_eq!(Severity::from_cvss_score(7.5), Severity::High);
        assert_eq!(Severity::from_cvss_score(5.0), Severity::Medium);
        assert_eq!(Severity::from_cvss_score(2.0), Severity::Low);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_severity_parse() {
        assert_eq!(Severity::parse("high"), Some(Severity::High));
        assert_eq!(Severity::parse("HIGH"), Some(Severity::High));
        assert_eq!(Severity::parse("moderate"), Some(Severity::Medium));
        assert_eq!(Severity::parse("unknown"), None);
    }

    #[test]
    fn test_vulnerability_info_is_vulnerable() {
        let info = VulnerabilityInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "npm".to_string(),
            vulnerabilities: vec![],
        };
        assert!(!info.is_vulnerable());

        let info_with_vuln = VulnerabilityInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "npm".to_string(),
            vulnerabilities: vec![Vulnerability {
                id: "TEST-001".to_string(),
                severity: Severity::High,
                score: Some(7.5),
                summary: "Test vulnerability".to_string(),
                url: None,
            }],
        };
        assert!(info_with_vuln.is_vulnerable());
    }

    #[test]
    fn test_max_severity() {
        let info = VulnerabilityInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "npm".to_string(),
            vulnerabilities: vec![
                Vulnerability {
                    id: "TEST-001".to_string(),
                    severity: Severity::Low,
                    score: Some(2.0),
                    summary: "Low".to_string(),
                    url: None,
                },
                Vulnerability {
                    id: "TEST-002".to_string(),
                    severity: Severity::High,
                    score: Some(7.5),
                    summary: "High".to_string(),
                    url: None,
                },
            ],
        };
        assert_eq!(info.max_severity(), Some(Severity::High));
    }

    #[test]
    fn test_filter_by_severity() {
        let info = VulnerabilityInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "npm".to_string(),
            vulnerabilities: vec![
                Vulnerability {
                    id: "TEST-001".to_string(),
                    severity: Severity::Low,
                    score: Some(2.0),
                    summary: "Low".to_string(),
                    url: None,
                },
                Vulnerability {
                    id: "TEST-002".to_string(),
                    severity: Severity::High,
                    score: Some(7.5),
                    summary: "High".to_string(),
                    url: None,
                },
            ],
        };
        let filtered = info.filter_by_severity(Severity::Medium);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "TEST-002");
    }
}
