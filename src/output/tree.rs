use super::{OutputFormat, OutputOptions};
use crate::error::Result;
use crate::graph::{DependencyGraph, QueryResult};
use crate::license::LicenseRisk;
use crate::security::VulnerabilityInfo;
use colored::Colorize;
use std::collections::HashMap;

pub struct TreeOutput;

impl OutputFormat for TreeOutput {
    fn format(&self, graph: &DependencyGraph, result: &QueryResult) -> Result<String> {
        self.format_with_security(graph, result, None)
    }
    
    fn format_with_security(
        &self,
        graph: &DependencyGraph,
        result: &QueryResult,
        vuln_info: Option<&VulnerabilityInfo>,
    ) -> Result<String> {
        let mut output = String::new();
        
        // Header: target@version with vulnerability marker
        let vuln_marker = if let Some(info) = vuln_info {
            if info.is_vulnerable() {
                format!(" {}", "[VULNERABLE]".red().bold())
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        output.push_str(&format!(
            "{}{}\n",
            format!("{}@{}", result.target_name, result.target_version)
                .yellow()
                .bold(),
            vuln_marker
        ));
        
        // Show vulnerability details if present
        if let Some(info) = vuln_info {
            for vuln in &info.vulnerabilities {
                let severity_str = match vuln.severity {
                    crate::security::Severity::Critical => format!("({})", vuln.severity).red().bold().to_string(),
                    crate::security::Severity::High => format!("({})", vuln.severity).red().to_string(),
                    crate::security::Severity::Medium => format!("({})", vuln.severity).yellow().to_string(),
                    crate::security::Severity::Low => format!("({})", vuln.severity).white().to_string(),
                };
                output.push_str(&format!(
                    "├── {}: {} {}\n",
                    "CVE".red().bold(),
                    vuln.id,
                    severity_str
                ));
                output.push_str(&format!(
                    "│   {}: {}\n",
                    "Summary".dimmed(),
                    vuln.summary.chars().take(80).collect::<String>()
                ));
                if let Some(ref url) = vuln.url {
                    output.push_str(&format!(
                        "│   {}: {}\n",
                        "Advisory".dimmed(),
                        url.blue().underline()
                    ));
                }
            }
        }
        
        if result.paths.is_empty() {
            output.push_str("\nNo paths found.\n");
            return Ok(output);
        }
        
        // Group paths by their direct dependent (first node)
        let mut paths_by_direct: HashMap<petgraph::graph::NodeIndex, Vec<&crate::graph::DependencyPath>> = HashMap::new();
        for path in &result.paths {
            if let Some(first) = path.direct_dependent() {
                paths_by_direct.entry(first).or_default().push(path);
            }
        }
        
        // Sort direct dependents by name for consistent output
        let mut direct_entries: Vec<_> = paths_by_direct.into_iter().collect();
        direct_entries.sort_by(|a, b| {
            let name_a = &graph.graph[a.0].name;
            let name_b = &graph.graph[b.0].name;
            name_a.cmp(name_b)
        });
        
        let total_directs = direct_entries.len();
        
        for (i, (direct_idx, paths)) in direct_entries.iter().enumerate() {
            let direct_pkg = &graph.graph[*direct_idx];
            let is_last_direct = i == total_directs - 1;
            let branch = if is_last_direct { "└──" } else { "├──" };
            
            // Check if any path through this direct dep is dev-only
            let is_dev = paths.iter().all(|p| p.is_dev());
            let dev_marker = if is_dev { " (dev)".dimmed().to_string() } else { String::new() };
            
            output.push_str(&format!(
                "{} Found via: {}{}\n",
                branch,
                format!("{}@{}", direct_pkg.name, direct_pkg.version).green(),
                dev_marker
            ));
            
            // Show the shortest path through this direct dependent
            if let Some(shortest) = paths.iter().min_by_key(|p| p.len()) {
                let continuation = if is_last_direct { "    " } else { "│   " };
                
                // Skip the first node (direct dependent) since we already showed it
                for (j, &node) in shortest.nodes.iter().enumerate().skip(1) {
                    let pkg = &graph.graph[node];
                    let is_target = node == result.target;
                    let is_last = j == shortest.nodes.len() - 1;
                    
                    let sub_branch = if is_last { "└── " } else { "├── " };
                    let sub_continuation = if is_last { "    " } else { "│   " };
                    
                    // Build indent based on depth
                    let indent = format!("{}{}", continuation, sub_continuation.repeat(j.saturating_sub(1)));
                    
                    let name_str = if is_target {
                        format!("{}@{}", pkg.name, pkg.version).yellow().bold().to_string()
                    } else {
                        format!("{}@{}", pkg.name, pkg.version)
                    };
                    
                    output.push_str(&format!("{}{}{}\n", indent, sub_branch, name_str));
                }
            }
        }
        
        // Summary line
        output.push_str(&format!(
            "\n{}: {} paths found (shortest: {}, longest: {})\n",
            "Summary".bold(),
            result.total_paths(),
            result.shortest_depth,
            result.longest_depth
        ));
        
        // Direct dependents
        let direct_names: Vec<String> = result.direct_dependents
            .iter()
            .map(|&idx| graph.graph[idx].name.clone())
            .collect();
        output.push_str(&format!(
            "{}: {}\n",
            "Direct dependents".bold(),
            direct_names.join(", ")
        ));
        
        Ok(output)
    }
    
    fn format_with_options(
        &self,
        graph: &DependencyGraph,
        result: &QueryResult,
        options: &OutputOptions,
    ) -> Result<String> {
        let mut output = String::new();
        
        // Get target package license info
        let target_license = if options.show_licenses {
            graph.graph[result.target].license.as_ref()
        } else {
            None
        };
        
        // Header: target@version with vulnerability and license markers
        let vuln_marker = if let Some(info) = options.vuln_info {
            if info.is_vulnerable() {
                format!(" {}", "[VULNERABLE]".red().bold())
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        let license_marker = if let Some(lic) = target_license {
            let risk_color = match lic.risk {
                LicenseRisk::High => format!(" [{}]", lic.spdx).red().bold().to_string(),
                LicenseRisk::Medium => format!(" [{}]", lic.spdx).yellow().to_string(),
                LicenseRisk::Low => format!(" [{}]", lic.spdx).green().to_string(),
                LicenseRisk::Unknown => format!(" [{}]", lic.spdx).dimmed().to_string(),
            };
            risk_color
        } else {
            String::new()
        };
        
        output.push_str(&format!(
            "{}{}{}\n",
            format!("{}@{}", result.target_name, result.target_version)
                .yellow()
                .bold(),
            vuln_marker,
            license_marker
        ));
        
        // Show vulnerability details if present
        if let Some(info) = options.vuln_info {
            for vuln in &info.vulnerabilities {
                let severity_str = match vuln.severity {
                    crate::security::Severity::Critical => format!("({})", vuln.severity).red().bold().to_string(),
                    crate::security::Severity::High => format!("({})", vuln.severity).red().to_string(),
                    crate::security::Severity::Medium => format!("({})", vuln.severity).yellow().to_string(),
                    crate::security::Severity::Low => format!("({})", vuln.severity).white().to_string(),
                };
                output.push_str(&format!(
                    "├── {}: {} {}\n",
                    "CVE".red().bold(),
                    vuln.id,
                    severity_str
                ));
                output.push_str(&format!(
                    "│   {}: {}\n",
                    "Summary".dimmed(),
                    vuln.summary.chars().take(80).collect::<String>()
                ));
            }
        }
        
        // Show license warning if copyleft
        if let Some(lic) = target_license {
            if lic.is_copyleft {
                let warning = match lic.risk {
                    LicenseRisk::High => format!("├── {}: Strong copyleft - viral license", "License".red().bold()),
                    LicenseRisk::Medium => format!("├── {}: Weak copyleft - review requirements", "License".yellow().bold()),
                    _ => String::new(),
                };
                if !warning.is_empty() {
                    output.push_str(&format!("{}\n", warning));
                }
            }
        }
        
        if result.paths.is_empty() {
            output.push_str("\nNo paths found.\n");
            return Ok(output);
        }
        
        // Group paths by their direct dependent
        let mut paths_by_direct: HashMap<petgraph::graph::NodeIndex, Vec<&crate::graph::DependencyPath>> = HashMap::new();
        for path in &result.paths {
            if let Some(first) = path.direct_dependent() {
                paths_by_direct.entry(first).or_default().push(path);
            }
        }
        
        let mut direct_entries: Vec<_> = paths_by_direct.into_iter().collect();
        direct_entries.sort_by(|a, b| {
            let name_a = &graph.graph[a.0].name;
            let name_b = &graph.graph[b.0].name;
            name_a.cmp(name_b)
        });
        
        let total_directs = direct_entries.len();
        
        for (i, (direct_idx, paths)) in direct_entries.iter().enumerate() {
            let direct_pkg = &graph.graph[*direct_idx];
            let is_last_direct = i == total_directs - 1;
            let branch = if is_last_direct { "└──" } else { "├──" };
            
            let is_dev = paths.iter().all(|p| p.is_dev());
            let dev_marker = if is_dev { " (dev)".dimmed().to_string() } else { String::new() };
            
            // Add license info for direct dependent if showing licenses
            let direct_license = if options.show_licenses {
                if let Some(lic) = &direct_pkg.license {
                    match lic.risk {
                        LicenseRisk::High => format!(" [{}]", lic.spdx).red().to_string(),
                        LicenseRisk::Medium => format!(" [{}]", lic.spdx).yellow().to_string(),
                        _ => String::new(),
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            
            output.push_str(&format!(
                "{} Found via: {}{}{}\n",
                branch,
                format!("{}@{}", direct_pkg.name, direct_pkg.version).green(),
                dev_marker,
                direct_license
            ));
            
            // Show shortest path
            if let Some(shortest) = paths.iter().min_by_key(|p| p.len()) {
                let continuation = if is_last_direct { "    " } else { "│   " };
                
                for (j, &node) in shortest.nodes.iter().enumerate().skip(1) {
                    let pkg = &graph.graph[node];
                    let is_target = node == result.target;
                    let is_last = j == shortest.nodes.len() - 1;
                    
                    let sub_branch = if is_last { "└── " } else { "├── " };
                    let sub_continuation = if is_last { "    " } else { "│   " };
                    let indent = format!("{}{}", continuation, sub_continuation.repeat(j.saturating_sub(1)));
                    
                    let name_str = if is_target {
                        format!("{}@{}", pkg.name, pkg.version).yellow().bold().to_string()
                    } else {
                        format!("{}@{}", pkg.name, pkg.version)
                    };
                    
                    // Add license marker for intermediate packages
                    let pkg_license = if options.show_licenses && !is_target {
                        if let Some(lic) = &pkg.license {
                            if lic.is_copyleft {
                                match lic.risk {
                                    LicenseRisk::High => format!(" [{}]", lic.spdx).red().to_string(),
                                    LicenseRisk::Medium => format!(" [{}]", lic.spdx).yellow().to_string(),
                                    _ => String::new(),
                                }
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    
                    output.push_str(&format!("{}{}{}{}\n", indent, sub_branch, name_str, pkg_license));
                }
            }
        }
        
        // Summary line
        output.push_str(&format!(
            "\n{}: {} paths found (shortest: {}, longest: {})\n",
            "Summary".bold(),
            result.total_paths(),
            result.shortest_depth,
            result.longest_depth
        ));
        
        // License summary if provided
        if let Some(summary) = options.license_summary {
            if summary.copyleft_count > 0 {
                output.push_str(&format!(
                    "{}: {} copyleft ({} high risk, {} medium risk)\n",
                    "Licenses".bold(),
                    summary.copyleft_count,
                    summary.high_risk.len(),
                    summary.medium_risk.len()
                ));
            }
        }
        
        // Direct dependents
        let direct_names: Vec<String> = result.direct_dependents
            .iter()
            .map(|&idx| graph.graph[idx].name.clone())
            .collect();
        output.push_str(&format!(
            "{}: {}\n",
            "Direct dependents".bold(),
            direct_names.join(", ")
        ));
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphBuilder, PathFinder};

    fn sample_graph_and_result() -> (DependencyGraph, QueryResult) {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let dep_a = builder.add_package("dep-a", "2.0.0");
        let target = builder.add_package("target", "3.0.0");
        
        builder.add_dep(root, dep_a);
        builder.add_dep(dep_a, target);
        
        let graph = builder.build();
        let target_idx = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        let result = finder.query(target_idx);
        
        (graph, result)
    }

    #[test]
    fn test_tree_output_header() {
        let (graph, result) = sample_graph_and_result();
        let formatter = TreeOutput;
        
        let output = formatter.format(&graph, &result).unwrap();
        assert!(output.contains("target@3.0.0"));
    }

    #[test]
    fn test_tree_output_found_via() {
        let (graph, result) = sample_graph_and_result();
        let formatter = TreeOutput;
        
        let output = formatter.format(&graph, &result).unwrap();
        assert!(output.contains("Found via:"));
        // "Found via:" shows the direct dependency (dep-a), not the root (myapp)
        assert!(output.contains("dep-a@2.0.0"));
    }

    #[test]
    fn test_tree_output_summary() {
        let (graph, result) = sample_graph_and_result();
        let formatter = TreeOutput;
        
        let output = formatter.format(&graph, &result).unwrap();
        assert!(output.contains("Summary:"));
        assert!(output.contains("paths found"));
        assert!(output.contains("Direct dependents:"));
    }

    #[test]
    fn test_tree_output_empty() {
        let graph = DependencyGraph::new();
        let result = QueryResult {
            target: petgraph::graph::NodeIndex::new(0),
            target_name: "missing".to_string(),
            target_version: "1.0.0".to_string(),
            paths: vec![],
            shortest_depth: 0,
            longest_depth: 0,
            direct_dependents: vec![],
        };
        let formatter = TreeOutput;
        
        let output = formatter.format(&graph, &result).unwrap();
        assert!(output.contains("No paths found"));
    }
}
