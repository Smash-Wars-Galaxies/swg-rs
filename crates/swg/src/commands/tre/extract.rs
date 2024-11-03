use clap::Args;
use miette::{Context, IntoDiagnostic, Result};
use std::{fs::File, path::PathBuf};
use swg_tre::{read::TreFile, TreArchive};
use tracing::info;

#[derive(Args)]
pub struct ExtractArgs {
    /// An input TRE file
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,

    /// A target directory
    #[arg(short, long, value_name = "DIR")]
    directory: PathBuf,

    /// Allow overwriting the target
    #[arg(long, default_value_t = false)]
    overwrite: bool,
}

impl ExtractArgs {
    pub fn handle(&self) -> Result<()> {
        let mut f = File::open(&self.file)
            .into_diagnostic()
            .context(format!("path: {}", &self.file.display()))?;
        let mut tre = TreArchive::new(&mut f)?;

        let count = tre.len();
        for i in 0..count {
            let mut f_tre: TreFile<'_, &mut File> = tre.by_index(i)?;

            let p = self.directory.join(f_tre.name());
            info!("writing {}", p.display());

            let _ = std::fs::create_dir_all(p.parent().unwrap());
            let mut out = if !self.overwrite {
                File::create_new(&p)
                    .into_diagnostic()
                    .context(format!("creating {}", &p.display()))?
            } else {
                File::create(&p)
                    .into_diagnostic()
                    .context(format!("creating {}", &p.display()))?
            };

            std::io::copy(&mut f_tre, &mut out).into_diagnostic()?;
        }
        Ok(())
    }
}
