use clap::Parser;
use colored::Colorize;
use integration_tests_sv2::start_tracing;

use interoperability_tests_sv2::scenario::{AppType, Scenario};
use interoperability_tests_sv2::setup_connection;

fn scenarios() -> Vec<Scenario> {
    vec![Scenario::new(
        "SetupConnection",
        "Happy-path and error-case SetupConnection handshakes",
        |endpoint, app_type| Box::pin(setup_connection::run(endpoint, app_type)),
    )]
}

#[derive(Parser)]
#[command(
    name = "sv2-compliance",
    about = "SV2 Interoperability Compliance Tester"
)]
struct Cli {
    /// SV2 endpoint address (host:port)
    #[arg(long)]
    endpoint: std::net::SocketAddr,

    /// Type of SV2 application: pool, job-declarator-server, template-provider
    #[arg(long, default_value = "pool")]
    app_type: AppType,

    /// Filter scenarios by name (partial match)
    #[arg(long)]
    scenario: Option<String>,

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

    let app_type = cli.app_type;

    println!(
        "Testing {} endpoint at {}",
        format!("{app_type:?}").blue().bold(),
        cli.endpoint.to_string().yellow()
    );
    println!();

    let all_scenarios = scenarios();
    let total_tests = all_scenarios.len();
    let mut total_passed = 0u32;
    let mut total_failed = 0u32;

    for scenario in &all_scenarios {
        // Filter by scenario name
        if let Some(filter) = &cli.scenario {
            if !scenario
                .name
                .to_lowercase()
                .contains(&filter.to_lowercase())
            {
                println!(
                    "  {} {} {}",
                    "⊘".dimmed(),
                    scenario.name.blue().bold(),
                    "(skipped)".dimmed()
                );
                continue;
            }
        }

        println!("  ── {} ──", scenario.name.blue().bold());
        println!("     {}", scenario.description.dimmed());
        println!();

        let results = scenario.execute(cli.endpoint, app_type).await;
        for result in &results {
            if result.passed {
                println!(
                    "    {} {} {}",
                    "✓".green(),
                    result.name,
                    if result.detail.is_empty() {
                        "".to_string()
                    } else {
                        format!("({})", result.detail.dimmed())
                    }
                );
                total_passed += 1;
            } else {
                println!("    {} {}: {}", "✗".red(), result.name, result.detail.red());
                total_failed += 1;
            }
        }
        println!();
    }

    // Summary
    let total = total_passed + total_failed;
    if total_failed == 0 {
        println!(
            "{} {}/{} tests passed across {total_tests} scenario(s)",
            "✓".green().bold(),
            total_passed.to_string().green().bold(),
            total
        );
    } else {
        println!(
            "{} {}/{} tests passed, {}/{} failed across {total_tests} scenario(s)",
            "✗".red().bold(),
            total_passed.to_string().green().bold(),
            total,
            total_failed.to_string().red().bold(),
            total
        );
        std::process::exit(1);
    }
}
