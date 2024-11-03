pub mod extract;
pub mod merge;

#[derive(clap::Subcommand)]
pub enum TreCommands {
    /// Extract a TRE file into a directory
    Extract(extract::ExtractArgs),
    /// Merge a directory into a TRE file
    Merge(merge::MergeArgs),
}

impl TreCommands {
    pub fn handle(&self) -> miette::Result<()> {
        match self {
            TreCommands::Extract(extract) => extract.handle(),
            TreCommands::Merge(merge) => merge.handle(),
        }
    }
}
