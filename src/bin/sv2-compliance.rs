use std::path::PathBuf;

use clap::Parser;
use colored::Colorize;
use integration_tests_sv2::start_tracing;
use interoperability_tests_sv2::{
    config::Config,
    runner::run_suite,
    scenarios::{ScenarioError, ScenarioReport},
};

#[derive(Parser)]
#[command(
    name = "sv2-compliance",
    about = "Sv2 interoperability compliance tester"
)]
struct Cli {
    /// Path to the TOML configuration file listing sites to test
    #[arg(short = 'c', long, default_value = "targets.toml")]
    config: PathBuf,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if cli.verbose {
        start_tracing();
    }

    let targets = match Config::from_path(&cli.config) {
        Ok(config) => config.targets(),
        Err(e) => {
            eprintln!("{} {e}", "error:".red().bold());
            std::process::exit(1);
        }
    };

    if targets.is_empty() {
        eprintln!("{} no endpoints configured to test", "error:".red().bold());
        std::process::exit(1);
    }

    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut not_exercised = 0u32;
    for target in &targets {
        println!(
            "── {} ──",
            format!(
                "{}: {:?} at {}",
                target.site_name, target.endpoint.app_type, target.endpoint.addr
            )
            .blue()
            .bold()
        );
        let reports = run_suite(&target.endpoint).await;
        for report in &reports {
            print_report(report);
            match &report.result {
                Ok(_) => passed += 1,
                Err(ScenarioError::NotExercised(_)) => not_exercised += 1,
                Err(ScenarioError::Failed(_)) => failed += 1,
            }
        }
        println!();
    }

    let total = passed + failed + not_exercised;
    let not_exercised_note = if not_exercised > 0 {
        format!(", {not_exercised} not exercised")
    } else {
        String::new()
    };
    if failed == 0 {
        println!(
            "{} {}/{} scenarios passed{} across {} target(s)",
            "✓".green().bold(),
            passed,
            total,
            not_exercised_note,
            targets.len()
        );
    } else {
        println!(
            "{} {}/{} scenarios passed, {} failed{} across {} target(s)",
            "✗".red().bold(),
            passed,
            total,
            failed,
            not_exercised_note,
            targets.len()
        );
        std::process::exit(1);
    }
}

fn print_report(report: &ScenarioReport) {
    match &report.result {
        Ok(detail) => {
            let detail = detail
                .as_deref()
                .map(|d| format!(" ({})", d.dimmed()))
                .unwrap_or_default();
            println!("  {} {}{detail}", "✓".green(), report.id.blue());
        }
        Err(ScenarioError::NotExercised(reason)) => println!(
            "  {} {}: {}",
            "○".dimmed(),
            report.id.blue(),
            format!("not exercised: {reason}").dimmed()
        ),
        Err(e) => println!(
            "  {} {}: {}",
            "✗".red(),
            report.id.blue(),
            e.to_string().red()
        ),
    }
}
