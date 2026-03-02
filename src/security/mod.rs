//! Security integration module - queries OSV database for vulnerabilities.

mod osv;
mod types;

pub use osv::OsvClient;
pub use types::{Severity, Vulnerability, VulnerabilityInfo};
