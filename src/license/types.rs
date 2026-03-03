use serde::Serialize;

/// License information for a package
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LicenseInfo {
    /// SPDX license identifier or license text
    pub spdx: String,
    /// Whether this is a copyleft license (GPL, AGPL, etc.)
    pub is_copyleft: bool,
    /// Risk level for commercial use
    pub risk: LicenseRisk,
}

/// Risk level for license compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum LicenseRisk {
    /// Permissive license (MIT, Apache, BSD, ISC, etc.)
    Low,
    /// Weak copyleft (LGPL, MPL)
    Medium,
    /// Strong copyleft (GPL, AGPL)
    High,
    /// Unknown or proprietary
    Unknown,
}

impl LicenseInfo {
    pub fn new(spdx: impl Into<String>) -> Self {
        let spdx = spdx.into();
        let normalized = normalize_license(&spdx);
        let (is_copyleft, risk) = classify_license(&normalized);
        
        Self {
            spdx,
            is_copyleft,
            risk,
        }
    }
    
    pub fn unknown() -> Self {
        Self {
            spdx: "UNKNOWN".to_string(),
            is_copyleft: false,
            risk: LicenseRisk::Unknown,
        }
    }
}

impl Default for LicenseInfo {
    fn default() -> Self {
        Self::unknown()
    }
}

/// Normalize license identifier for comparison
fn normalize_license(license: &str) -> String {
    license
        .trim()
        .to_uppercase()
        .replace(['-', ' '], "")
}

/// Classify license as copyleft and risk level
fn classify_license(normalized: &str) -> (bool, LicenseRisk) {
    // Check weak copyleft FIRST (LGPL before GPL, since LGPL contains GPL)
    let weak_copyleft = [
        "LGPL", "LGPL2", "LGPL2.1", "LGPL3", "LGPL3.0",
        "LGPLV2", "LGPLV2.1", "LGPLV3",
        "LGPL2.0ONLY", "LGPL2.1ONLY", "LGPL3.0ONLY",
        "LGPL2.0ORLATER", "LGPL2.1ORLATER", "LGPL3.0ORLATER",
        "MPL", "MPL1.0", "MPL1.1", "MPL2.0",
        "EPL", "EPL1.0", "EPL2.0",
        "CPL", "CPL1.0",
        "CDDL", "CDDL1.0", "CDDL1.1",
    ];
    
    // Check weak copyleft first (LGPL before GPL)
    for copyleft in weak_copyleft {
        if normalized.contains(copyleft) {
            return (true, LicenseRisk::Medium);
        }
    }
    
    // Strong copyleft (high risk for commercial)
    let strong_copyleft = [
        "GPL", "GPL2", "GPL3", "GPL2.0", "GPL3.0",
        "GPLV2", "GPLV3", "GPL2.0ONLY", "GPL3.0ONLY",
        "GPL2.0ORLATER", "GPL3.0ORLATER",
        "AGPL", "AGPL3", "AGPL3.0", "AGPLV3",
        "AGPL3.0ONLY", "AGPL3.0ORLATER",
        "SSPL", "SSPL1.0",
    ];
    
    // Check strong copyleft
    for copyleft in strong_copyleft {
        if normalized.contains(copyleft) {
            return (true, LicenseRisk::High);
        }
    }
    
    // Permissive (low risk)
    let permissive = [
        "MIT", "ISC", "BSD", "BSD2CLAUSE", "BSD3CLAUSE",
        "APACHE", "APACHE2", "APACHE2.0",
        "CC0", "CC01.0", "UNLICENSE", "WTFPL",
        "ZLIB", "0BSD", "BSDLIKE",
        "PUBLICDOMAIN", "CC0", "CC01.0UNIVERSAL",
    ];
    
    // Check permissive
    for perm in permissive {
        if normalized.contains(perm) {
            return (false, LicenseRisk::Low);
        }
    }
    
    // Unknown
    (false, LicenseRisk::Unknown)
}

/// Summary of licenses in a project
#[derive(Debug, Clone, Serialize)]
pub struct LicenseSummary {
    pub total_packages: usize,
    pub copyleft_count: usize,
    pub permissive_count: usize,
    pub unknown_count: usize,
    pub high_risk: Vec<PackageLicense>,
    pub medium_risk: Vec<PackageLicense>,
}

/// Package with license info for summary
#[derive(Debug, Clone, Serialize)]
pub struct PackageLicense {
    pub name: String,
    pub version: String,
    pub license: String,
}

impl LicenseSummary {
    pub fn new() -> Self {
        Self {
            total_packages: 0,
            copyleft_count: 0,
            permissive_count: 0,
            unknown_count: 0,
            high_risk: Vec::new(),
            medium_risk: Vec::new(),
        }
    }
    
    pub fn add(&mut self, name: &str, version: &str, license: &LicenseInfo) {
        self.total_packages += 1;
        
        match license.risk {
            LicenseRisk::High => {
                self.copyleft_count += 1;
                self.high_risk.push(PackageLicense {
                    name: name.to_string(),
                    version: version.to_string(),
                    license: license.spdx.clone(),
                });
            }
            LicenseRisk::Medium => {
                self.copyleft_count += 1;
                self.medium_risk.push(PackageLicense {
                    name: name.to_string(),
                    version: version.to_string(),
                    license: license.spdx.clone(),
                });
            }
            LicenseRisk::Low => {
                self.permissive_count += 1;
            }
            LicenseRisk::Unknown => {
                self.unknown_count += 1;
            }
        }
    }
}

impl Default for LicenseSummary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mit_is_permissive() {
        let license = LicenseInfo::new("MIT");
        assert!(!license.is_copyleft);
        assert_eq!(license.risk, LicenseRisk::Low);
    }

    #[test]
    fn test_apache_is_permissive() {
        let license = LicenseInfo::new("Apache-2.0");
        assert!(!license.is_copyleft);
        assert_eq!(license.risk, LicenseRisk::Low);
    }

    #[test]
    fn test_isc_is_permissive() {
        let license = LicenseInfo::new("ISC");
        assert!(!license.is_copyleft);
        assert_eq!(license.risk, LicenseRisk::Low);
    }

    #[test]
    fn test_bsd_is_permissive() {
        let license = LicenseInfo::new("BSD-3-Clause");
        assert!(!license.is_copyleft);
        assert_eq!(license.risk, LicenseRisk::Low);
    }

    #[test]
    fn test_gpl_is_copyleft() {
        let license = LicenseInfo::new("GPL-3.0");
        assert!(license.is_copyleft);
        assert_eq!(license.risk, LicenseRisk::High);
    }

    #[test]
    fn test_agpl_is_copyleft() {
        let license = LicenseInfo::new("AGPL-3.0-only");
        assert!(license.is_copyleft);
        assert_eq!(license.risk, LicenseRisk::High);
    }

    #[test]
    fn test_lgpl_is_weak_copyleft() {
        let license = LicenseInfo::new("LGPL-2.1");
        assert!(license.is_copyleft);
        assert_eq!(license.risk, LicenseRisk::Medium);
    }

    #[test]
    fn test_mpl_is_weak_copyleft() {
        let license = LicenseInfo::new("MPL-2.0");
        assert!(license.is_copyleft);
        assert_eq!(license.risk, LicenseRisk::Medium);
    }

    #[test]
    fn test_unknown_license() {
        let license = LicenseInfo::new("Proprietary");
        assert!(!license.is_copyleft);
        assert_eq!(license.risk, LicenseRisk::Unknown);
    }

    #[test]
    fn test_default_is_unknown() {
        let license = LicenseInfo::default();
        assert_eq!(license.spdx, "UNKNOWN");
        assert_eq!(license.risk, LicenseRisk::Unknown);
    }

    #[test]
    fn test_license_summary() {
        let mut summary = LicenseSummary::new();
        
        summary.add("lodash", "4.17.21", &LicenseInfo::new("MIT"));
        summary.add("express", "4.18.2", &LicenseInfo::new("MIT"));
        summary.add("some-gpl", "1.0.0", &LicenseInfo::new("GPL-3.0"));
        summary.add("some-lgpl", "1.0.0", &LicenseInfo::new("LGPL-2.1"));
        
        assert_eq!(summary.total_packages, 4);
        assert_eq!(summary.permissive_count, 2);
        assert_eq!(summary.copyleft_count, 2);
        assert_eq!(summary.high_risk.len(), 1);
        assert_eq!(summary.medium_risk.len(), 1);
    }
}
