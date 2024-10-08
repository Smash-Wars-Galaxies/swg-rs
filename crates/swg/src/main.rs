use clap::{Parser, Subcommand};
use miette::miette;
use miette::{Context, IntoDiagnostic, Result};
use std::{
    fs::File,
    io::{self},
    path::PathBuf,
};
use swg_tre::compression::CompressionMethod;
use swg_tre::read::{TreArchive, TreFile};
use swg_tre::write::{TreWriter, TreWriterOptions};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Handle TRE files
    Tre {
        #[command(subcommand)]
        command: TreCommands,
    },
}

#[derive(Subcommand)]
enum TreCommands {
    /// Extract a TRE file into a directory
    Extract {
        /// An input TRE file
        #[arg(short, long, value_name = "FILE")]
        file: PathBuf,

        /// A target directory
        #[arg(short, long, value_name = "DIR")]
        directory: PathBuf,

        /// Allow overwriting the target
        #[arg(long, default_value_t = false)]
        overwrite: bool,
    },
    /// Merge a directory into a TRE file
    Merge {
        /// An input directory
        #[arg(short, long, value_name = "DIR")]
        directory: PathBuf,

        /// A target TRE file
        #[arg(short, long, value_name = "FILE")]
        file: PathBuf,

        /// Allow overwriting the target
        #[arg(long, default_value_t = false)]
        overwrite: bool,
    },
}

fn main_tre_extract(file: &PathBuf, output: PathBuf, overwrite: bool) -> Result<()> {
    let mut f = File::open(file)
        .into_diagnostic()
        .context(format!("path: {}", &file.display()))?;
    let mut tre = TreArchive::new(&mut f)?;

    let count = tre.len();
    for i in 0..count {
        let mut f_tre: TreFile<'_, &mut File> = tre.by_index(i)?;

        let p = output.join(f_tre.name());
        info!("writing {}", p.display());

        let _ = std::fs::create_dir_all(p.parent().unwrap());
        let mut out = if !overwrite {
            File::create_new(&p)
                .into_diagnostic()
                .context(format!("creating {}", &p.display()))?
        } else {
            File::create(&p)
                .into_diagnostic()
                .context(format!("creating {}", &p.display()))?
        };

        io::copy(&mut f_tre, &mut out).into_diagnostic()?;
    }
    Ok(())
}

fn main_tre_merge(directory: &PathBuf, target: &PathBuf, overwrite: bool) -> Result<()> {
    info!("creating {}", target.display());

    let files = WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .collect::<Vec<_>>();

    if files.is_empty() {
        return Err(miette!("directory is empty"));
    }

    let mut out = if !overwrite {
        File::create_new(target)
            .into_diagnostic()
            .context(format!("creating {}", target.display()))?
    } else {
        File::create(target)
            .into_diagnostic()
            .context(format!("creating {}", target.display()))?
    };

    let mut tre = TreWriter::new(
        &mut out,
        TreWriterOptions::builder()
            .name_compression(CompressionMethod::Zlib)
            .record_compression(CompressionMethod::Zlib)
            .build(),
    );

    for file in files {
        let name = file.path().strip_prefix(directory).into_diagnostic()?;
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

fn main_tre(command: &TreCommands) -> Result<()> {
    match command {
        TreCommands::Extract {
            file,
            directory,
            overwrite,
        } => main_tre_extract(file, directory.to_path_buf(), *overwrite)?,
        TreCommands::Merge {
            directory,
            file,
            overwrite,
        } => main_tre_merge(directory, file, *overwrite)?,
    }
    Ok(())
}

fn main() -> miette::Result<()> {
    better_panic::install();

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();
    match &cli.command {
        Commands::Tre { command } => main_tre(command)?,
    }

    Ok(())
}
