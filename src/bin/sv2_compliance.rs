use std::net::SocketAddr;

use clap::Parser;
use colored::Colorize;
use integration_tests_sv2::start_tracing;

use interoperability_tests_sv2::config::Config;
use interoperability_tests_sv2::runner::{build_targets, print_summary, run_targets, Target};
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
    #[arg(long, required_unless_present = "config", conflicts_with = "config")]
    endpoint: Option<SocketAddr>,

    /// Type of SV2 application: pool, job-declarator-server, template-provider
    #[arg(long, default_value = "pool")]
    app_type: AppType,

    /// Path to a TOML config file listing sites to test
    #[arg(long)]
    config: Option<String>,

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

    let all_scenarios = scenarios();

    let targets = if let Some(config_path) = &cli.config {
        let config = match Config::from_path(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("{} {e}", "error:".red().bold());
                std::process::exit(1);
            }
        };
        build_targets(&config)
    } else {
        let endpoint = cli.endpoint.expect("endpoint required without --config");
        vec![Target {
            site_name: endpoint.to_string(),
            endpoint,
            app_type: cli.app_type,
            scenarios: None,
        }]
    };

    if targets.is_empty() {
        eprintln!("{} no endpoints configured to test", "error:".red().bold());
        std::process::exit(1);
    }

    println!(
        "Running {} scenario(s) against {} target(s)",
        all_scenarios.len().to_string().blue().bold(),
        targets.len().to_string().yellow().bold()
    );
    println!();

    let summary = run_targets(&targets, cli.scenario.as_deref(), &all_scenarios).await;

    println!();
    print_summary(&summary, targets.len());
    if summary.failed > 0 {
        std::process::exit(1);
    }
}
