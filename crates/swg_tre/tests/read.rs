use std::path::PathBuf;
use std::{fs::File, io::Read};
use swg_tre::{error::Error, read::TreArchive};
use tracing::info;
use tracing_test::traced_test;
use walkdir::WalkDir;

fn validate_tre(path: &PathBuf) -> Result<(), Error> {
    info!("testing {}", &path.display());

    let parent_dir = &path
        .parent()
        .ok_or(Error::CustomError("unable to find parent".into()))?
        .join(
            path.file_stem()
                .ok_or(Error::CustomError("unable to create file stem".into()))?,
        );

    info!("comparing to files in {}", parent_dir.display());

    let expected_files = WalkDir::new(parent_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .collect::<Vec<_>>();

    let mut r = File::open(path)?;
    let mut tre = TreArchive::new(&mut r)?;
    assert_eq!(tre.len(), expected_files.len());

    let count = tre.len();
    for i in 0..count {
        let mut f_tre = tre.by_index(i)?;

        let p = parent_dir.join(f_tre.name());
        info!("comparing to {}", p.display());

        let mut expected = Vec::new();
        let mut f_real = File::open(&p)?;
        f_real.read_to_end(&mut expected)?;

        let mut actual = Vec::new();
        f_tre.read_to_end(&mut actual)?;

        assert_eq!(expected.len(), actual.len());
        assert_eq!(expected, actual);
    }

    Ok(())
}

#[traced_test]
#[test]
fn validate_tre_parsing() -> Result<(), Error> {
    let to_test = std::fs::read_dir(format!("{}/resources/", env!("CARGO_MANIFEST_DIR")))?
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
        validate_tre(&path)?;
    }

    Ok(())
}
