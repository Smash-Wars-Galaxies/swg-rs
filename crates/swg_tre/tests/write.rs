use miette::{IntoDiagnostic, Result};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use swg_tre::{
    error::Error,
    read::TreArchive,
    write::{TreWriter, TreWriterOptions},
};
use tracing::{info, instrument};
use tracing_test::traced_test;

#[instrument(skip_all, fields(file=%path.file_name().unwrap().to_string_lossy()))]
fn validate_tre_merge(path: &Path) -> Result<()> {
    let input_file = File::open(path).into_diagnostic()?;
    let mut tre_input = TreArchive::new(&input_file)?;

    let parent_dir = &path
        .parent()
        .ok_or(Error::CustomError("unable to find parent".into()))?
        .join(
            path.file_stem()
                .ok_or(Error::CustomError("unable to create file stem".into()))?,
        );

    let mut tre = TreWriter::new(
        std::io::Cursor::new(Vec::new()),
        TreWriterOptions::builder()
            .name_compression(tre_input.get_name_compression())
            .record_compression(tre_input.get_record_compression())
            .build(),
    );

    let mut buffer = Vec::new();
    for i in 0..tre_input.len() {
        let expected_tre_file = tre_input.by_index(i)?;
        info!("inserting {:#?}", &expected_tre_file.name());

        tre.start_file(
            expected_tre_file.name(),
            expected_tre_file.compression_method(),
        )?;
        File::open(parent_dir.join(expected_tre_file.name()))
            .into_diagnostic()?
            .read_to_end(&mut buffer)
            .into_diagnostic()?;
        tre.write_all(&buffer).into_diagnostic()?;
        buffer.clear();
    }

    let mut actual = tre.finish()?;

    // OpenOptions::new()
    //     .truncate(true)
    //     .write(true)
    //     .create(true)
    //     .open(path.with_extension("generated.tre"))
    //     .into_diagnostic()?
    //     .write_all(actual.get_ref())
    //     .into_diagnostic()?;

    // Due to differences in zlib implementations we can't test outputs generated with different compressors
    // Instead we will open the file and validate that the results match our expectation

    // Rewind so we can read from the generated data
    actual.rewind().into_diagnostic()?;

    let mut tre_output = TreArchive::new(actual)?;

    assert_eq!(tre_input.len(), tre_output.len());
    assert_eq!(
        tre_input.file_names().collect::<Vec<_>>(),
        tre_output.file_names().collect::<Vec<_>>()
    );

    for i in 0..tre_input.len() {
        let mut input_buffer = Vec::new();
        let mut expected_tre_file = tre_input.by_index(i)?;
        expected_tre_file
            .read_to_end(&mut input_buffer)
            .into_diagnostic()?;

        info!("comparing {}", expected_tre_file.name());

        let mut output_buffer = Vec::new();
        let mut actual_tre_file = tre_output.by_name(expected_tre_file.name())?;
        actual_tre_file
            .read_to_end(&mut output_buffer)
            .into_diagnostic()?;

        assert_eq!(input_buffer.len(), output_buffer.len());
    }

    Ok(())
}

#[traced_test]
#[test]
fn merge_tre() -> Result<()> {
    let to_test = std::fs::read_dir(format!("{}/resources/", env!("CARGO_MANIFEST_DIR")))
        .into_diagnostic()?
        // Filter out all those directory entries which couldn't be read
        .filter_map(|res| res.ok())
        // Map the directory entries to paths
        .map(|dir_entry| dir_entry.path())
        .filter(|e| e.is_file())
        .filter(|path| {
            path.file_name().map_or(false, |name| {
                name.to_str().map_or(false, |f| {
                    f.ends_with(".tre") && !f.ends_with(".generated.tre")
                })
            })
        });

    for path in to_test {
        validate_tre_merge(&path)?;
    }

    Ok(())
}
