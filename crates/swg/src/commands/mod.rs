pub mod tre;

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Handle TRE files
    Tre {
        #[command(subcommand)]
        command: tre::TreCommands,
    },
}

impl Commands {
    pub fn handle(&self) -> miette::Result<()> {
        match self {
            Commands::Tre { command } => command.handle(),
        }
    }
}
