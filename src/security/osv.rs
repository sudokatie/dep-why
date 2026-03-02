//! OSV (Open Source Vulnerabilities) API client.

use crate::error::Error;
use super::types::{Severity, Vulnerability, VulnerabilityInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const OSV_API_URL: &str = "https://api.osv.dev/v1/query";
const CACHE_TTL_SECS: u64 = 3600; // 1 hour

/// OSV API client with caching.
pub struct OsvClient {
    cache_dir: PathBuf,
    cache_enabled: bool,
}

impl OsvClient {
    /// Create a new OSV client.
    pub fn new() -> Self {
        let cache_dir = dirs_next::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dep-why")
            .join("osv");
        
        Self {
            cache_dir,
            cache_enabled: true,
        }
    }

    /// Create client without caching (for testing).
    pub fn without_cache() -> Self {
        Self {
            cache_dir: PathBuf::new(),
            cache_enabled: false,
        }
    }

    /// Check a package for vulnerabilities.
    pub fn check_package(
        &self,
        name: &str,
        version: &str,
        ecosystem: &str,
    ) -> Result<VulnerabilityInfo, Error> {
        // Check cache first
        if self.cache_enabled {
            if let Some(cached) = self.get_cached(name, version, ecosystem) {
                return Ok(cached);
            }
        }

        // Query OSV API
        let vulnerabilities = self.query_osv(name, version, ecosystem)?;

        let info = VulnerabilityInfo {
            name: name.to_string(),
            version: version.to_string(),
            ecosystem: ecosystem.to_string(),
            vulnerabilities,
        };

        // Cache the result
        if self.cache_enabled {
            let _ = self.cache_result(&info);
        }

        Ok(info)
    }

    /// Query the OSV API.
    fn query_osv(
        &self,
        name: &str,
        version: &str,
        ecosystem: &str,
    ) -> Result<Vec<Vulnerability>, Error> {
        let osv_ecosystem = match ecosystem {
            "npm" => "npm",
            "cargo" => "crates.io",
            "pip" | "pypi" => "PyPI",
            _ => ecosystem,
        };

        let request = OsvRequest {
            package: OsvPackage {
                name: name.to_string(),
                ecosystem: osv_ecosystem.to_string(),
            },
            version: version.to_string(),
        };

        // Use ureq for HTTP requests (blocking, simpler than reqwest)
        let response = ureq::post(OSV_API_URL)
            .set("Content-Type", "application/json")
            .send_json(&request);

        match response {
            Ok(resp) => {
                let osv_response: OsvResponse = resp.into_json()
                    .map_err(|e| Error::ParseError {
                        path: std::path::PathBuf::from("OSV API response"),
                        message: format!("Failed to parse: {}", e),
                    })?;
                
                Ok(osv_response.vulns.into_iter().map(|v| v.into()).collect())
            }
            Err(ureq::Error::Status(404, _)) => {
                // No vulnerabilities found
                Ok(vec![])
            }
            Err(e) => {
                // Log warning but don't fail - security check is optional
                eprintln!("Warning: OSV API request failed: {}", e);
                Ok(vec![])
            }
        }
    }

    /// Get cache file path.
    fn cache_path(&self, name: &str, version: &str, ecosystem: &str) -> PathBuf {
        // Sanitize name for filesystem
        let safe_name = name.replace(['/', '\\'], "_");
        self.cache_dir
            .join(ecosystem)
            .join(format!("{}@{}.json", safe_name, version))
    }

    /// Get cached result if valid.
    fn get_cached(&self, name: &str, version: &str, ecosystem: &str) -> Option<VulnerabilityInfo> {
        let path = self.cache_path(name, version, ecosystem);
        
        // Check if cache file exists and is recent
        let metadata = fs::metadata(&path).ok()?;
        let modified = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(modified).ok()?;
        
        if age > Duration::from_secs(CACHE_TTL_SECS) {
            return None;
        }

        // Read and parse cache
        let contents = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Cache a result.
    fn cache_result(&self, info: &VulnerabilityInfo) -> Result<(), std::io::Error> {
        let path = self.cache_path(&info.name, &info.version, &info.ecosystem);
        
        // Create cache directory
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write cache file
        let json = serde_json::to_string_pretty(info)?;
        fs::write(&path, json)
    }

    /// Check multiple packages in batch.
    pub fn check_packages(
        &self,
        packages: &[(String, String, String)], // (name, version, ecosystem)
    ) -> HashMap<String, VulnerabilityInfo> {
        let mut results = HashMap::new();

        for (name, version, ecosystem) in packages {
            let key = format!("{}@{}", name, version);
            match self.check_package(name, version, ecosystem) {
                Ok(info) => {
                    results.insert(key, info);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to check {}: {}", key, e);
                }
            }
        }

        results
    }
}

impl Default for OsvClient {
    fn default() -> Self {
        Self::new()
    }
}

// OSV API request/response types

#[derive(Serialize)]
struct OsvRequest {
    package: OsvPackage,
    version: String,
}

#[derive(Serialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}

#[derive(Deserialize)]
struct OsvResponse {
    #[serde(default)]
    vulns: Vec<OsvVulnerability>,
}

#[derive(Deserialize)]
struct OsvVulnerability {
    id: String,
    summary: Option<String>,
    details: Option<String>,
    #[serde(default)]
    severity: Vec<OsvSeverity>,
    #[serde(default)]
    references: Vec<OsvReference>,
}

#[derive(Deserialize)]
struct OsvSeverity {
    #[serde(rename = "type")]
    severity_type: String,
    score: String,
}

#[derive(Deserialize)]
struct OsvReference {
    #[serde(rename = "type")]
    ref_type: String,
    url: String,
}

impl From<OsvVulnerability> for Vulnerability {
    fn from(osv: OsvVulnerability) -> Self {
        // Parse severity from CVSS score
        let (severity, score) = osv.severity
            .iter()
            .find(|s| s.severity_type == "CVSS_V3")
            .and_then(|s| s.score.parse::<f64>().ok())
            .map(|score| (Severity::from_cvss_score(score), Some(score)))
            .unwrap_or((Severity::Medium, None));

        // Get advisory URL
        let url = osv.references
            .iter()
            .find(|r| r.ref_type == "ADVISORY")
            .map(|r| r.url.clone())
            .or_else(|| {
                // Fall back to GitHub advisory URL format
                if osv.id.starts_with("GHSA-") {
                    Some(format!("https://github.com/advisories/{}", osv.id))
                } else {
                    None
                }
            });

        // Get summary
        let summary = osv.summary
            .or(osv.details.map(|d| d.chars().take(200).collect()))
            .unwrap_or_else(|| "No description available".to_string());

        Vulnerability {
            id: osv.id,
            severity,
            score,
            summary,
            url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osv_vulnerability_conversion() {
        let osv = OsvVulnerability {
            id: "GHSA-test-1234".to_string(),
            summary: Some("Test vulnerability".to_string()),
            details: None,
            severity: vec![OsvSeverity {
                severity_type: "CVSS_V3".to_string(),
                score: "7.5".to_string(),
            }],
            references: vec![],
        };

        let vuln: Vulnerability = osv.into();
        assert_eq!(vuln.id, "GHSA-test-1234");
        assert_eq!(vuln.severity, Severity::High);
        assert_eq!(vuln.score, Some(7.5));
        assert_eq!(vuln.url, Some("https://github.com/advisories/GHSA-test-1234".to_string()));
    }

    #[test]
    fn test_cache_path_sanitization() {
        let client = OsvClient::new();
        let path = client.cache_path("@scope/package", "1.0.0", "npm");
        // The filename should have the slash replaced
        let filename = path.file_name().unwrap().to_string_lossy();
        assert!(!filename.contains('/'));
        assert!(filename.contains("@scope_package"));
    }
}
