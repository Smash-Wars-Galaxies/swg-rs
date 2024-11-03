use std::io::IsTerminal;

use clap::Parser;
use miette::{IntoDiagnostic, Result};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: swg::commands::Commands,
}

fn main() -> Result<()> {
    better_panic::install();

    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(std::io::stdout().is_terminal())
                .with_file(true)
                .with_line_number(true)
                .with_target(false)
                .without_time()
                .compact(),
        )
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
        )
        .try_init().into_diagnostic()?;

    cli.command.handle()
}
