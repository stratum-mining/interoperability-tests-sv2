use std::net::SocketAddr;

use colored::Colorize;

use crate::config::Config;
use crate::scenario::{AppType, Scenario};

/// A single endpoint to test, derived from either the CLI args or a config site.
pub struct Target {
    pub site_name: String,
    pub endpoint: SocketAddr,
    pub app_type: AppType,
    /// Per-site scenario allowlist (partial, case-insensitive match).
    pub scenarios: Option<Vec<String>>,
}

pub struct RunSummary {
    pub passed: u32,
    pub failed: u32,
}

/// Builds one [`Target`] per configured address: `pool_address` becomes a
/// `Pool` target and `jds_address` becomes a `JobDeclaratorServer` target.
pub fn build_targets(config: &Config) -> Vec<Target> {
    let mut targets = Vec::new();
    for site in &config.site {
        if let Some(addr) = site.pool_address {
            targets.push(Target {
                site_name: site.name.clone(),
                endpoint: addr,
                app_type: AppType::Pool,
                scenarios: site.scenarios.clone(),
            });
        }
        if let Some(addr) = site.jds_address {
            targets.push(Target {
                site_name: site.name.clone(),
                endpoint: addr,
                app_type: AppType::JobDeclaratorServer,
                scenarios: site.scenarios.clone(),
            });
        }
    }
    targets
}

/// Runs every scenario against every target and prints per-target results.
pub async fn run_targets(
    targets: &[Target],
    global_filter: Option<&str>,
    all_scenarios: &[Scenario],
) -> RunSummary {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut current_site: Option<&str> = None;

    for target in targets {
        if current_site != Some(target.site_name.as_str()) {
            if current_site.is_some() {
                println!();
            }
            println!("  ── {} ──", target.site_name.blue().bold());
            current_site = Some(target.site_name.as_str());
        }
        println!(
            "     testing {} endpoint at {}",
            format!("{:?}", target.app_type).blue(),
            target.endpoint.to_string().yellow()
        );

        for scenario in all_scenarios {
            // Global --scenario filter
            if let Some(filter) = global_filter {
                if !scenario_matches(filter, scenario.name) {
                    println!(
                        "    {} {} {}",
                        "⊘".dimmed(),
                        scenario.name.blue().bold(),
                        "(skipped by --scenario)".dimmed()
                    );
                    continue;
                }
            }
            // Per-site scenarios allowlist
            if let Some(allowlist) = &target.scenarios {
                let matched = allowlist
                    .iter()
                    .any(|entry| scenario_matches(entry, scenario.name));
                if !matched {
                    println!(
                        "    {} {} {}",
                        "⊘".dimmed(),
                        scenario.name.blue().bold(),
                        "(skipped by site config)".dimmed()
                    );
                    continue;
                }
            }

            println!("    ── {} ──", scenario.name.blue().bold());
            println!("       {}", scenario.description.dimmed());

            let results = scenario.execute(target.endpoint, target.app_type).await;
            for result in &results {
                if result.passed {
                    println!(
                        "      {} {} {}",
                        "✓".green(),
                        result.name,
                        if result.detail.is_empty() {
                            "".to_string()
                        } else {
                            format!("({})", result.detail.dimmed())
                        }
                    );
                    passed += 1;
                } else {
                    println!(
                        "      {} {}: {}",
                        "✗".red(),
                        result.name,
                        result.detail.red()
                    );
                    failed += 1;
                }
            }
            println!();
        }
    }

    RunSummary { passed, failed }
}

pub fn print_summary(summary: &RunSummary, target_count: usize) {
    let total = summary.passed + summary.failed;
    if summary.failed == 0 {
        println!(
            "{} {}/{} tests passed across {target_count} target(s)",
            "✓".green().bold(),
            summary.passed.to_string().green().bold(),
            total
        );
    } else {
        println!(
            "{} {}/{} tests passed, {}/{} failed across {target_count} target(s)",
            "✗".red().bold(),
            summary.passed.to_string().green().bold(),
            total,
            summary.failed.to_string().red().bold(),
            total
        );
    }
}

fn scenario_matches(query: &str, name: &str) -> bool {
    name.to_lowercase().contains(&query.to_lowercase())
}
