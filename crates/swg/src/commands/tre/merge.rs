use clap::Args;
use miette::miette;
use miette::{Context, IntoDiagnostic, Result};
use std::{fs::File, path::PathBuf};
use swg_tre::{write::TreWriterOptions, CompressionMethod, TreWriter};
use tracing::info;
use walkdir::WalkDir;

#[derive(Args)]
pub struct MergeArgs {
    /// An input directory
    #[arg(short, long, value_name = "DIR")]
    directory: PathBuf,

    /// A target TRE file
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,

    /// Allow overwriting the target
    #[arg(long, default_value_t = false)]
    overwrite: bool,
}

impl MergeArgs {
    pub fn handle(&self) -> Result<()> {
        info!("creating {}", &self.file.display());

        let files = WalkDir::new(&self.directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_type().is_dir())
            .collect::<Vec<_>>();

        if files.is_empty() {
            return Err(miette!("directory is empty"));
        }

        let mut out = if !self.overwrite {
            File::create_new(&self.file)
                .into_diagnostic()
                .context(format!("creating {}", &self.file.display()))?
        } else {
            File::create(&self.file)
                .into_diagnostic()
                .context(format!("creating {}", &self.file.display()))?
        };

        let mut tre = TreWriter::new(
            &mut out,
            TreWriterOptions::builder()
                .name_compression(CompressionMethod::Zlib)
                .record_compression(CompressionMethod::Zlib)
                .build(),
        );

        for file in files {
            let name = file
                .path()
                .strip_prefix(&self.directory)
                .into_diagnostic()?;
            info!("merging {}", name.display());

            tre.start_file(
                name.to_str()
                    .ok_or(miette!("unable to convert {} to a string", name.display()))?,
                CompressionMethod::Zlib,
            )
            .context(format!("starting entry for {}", name.display()))?;

            let mut f = File::open(file.path())
                .into_diagnostic()
                .context(format!("opening {}", file.path().display()))?;

            std::io::copy(&mut f, &mut tre)
                .into_diagnostic()
                .context(format!("copying {}", file.path().display()))?;
        }

        tre.finish().context("finalizing tre file")?;

        Ok(())
    }
}
