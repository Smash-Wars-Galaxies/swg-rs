use std::io::IsTerminal;

use clap::Parser;
use clap_verbosity_flag::InfoLevel;
use miette::{IntoDiagnostic, Result};
use tracing_log::AsTrace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod commands;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity<InfoLevel>,

    #[command(subcommand)]
    command: commands::Commands,
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
                .with_default_directive(cli.verbose.log_level_filter().as_trace().into())
                .from_env_lossy(),
        )
        .try_init()
        .into_diagnostic()?;

    cli.command.handle()
}
