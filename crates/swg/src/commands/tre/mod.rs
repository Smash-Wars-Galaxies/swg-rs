pub mod diff;
pub mod extract;
pub mod merge;

#[derive(clap::Subcommand)]
pub enum TreCommands {
    /// Compare Two TRE files
    Diff(diff::DiffArgs),
    /// Extract a TRE file into a directory
    Extract(extract::ExtractArgs),
    /// Merge a directory into a TRE file
    Merge(merge::MergeArgs),
}

impl TreCommands {
    pub fn handle(&self) -> miette::Result<()> {
        match self {
            TreCommands::Diff(diff) => diff.handle(),
            TreCommands::Extract(extract) => extract.handle(),
            TreCommands::Merge(merge) => merge.handle(),
        }
    }
}
