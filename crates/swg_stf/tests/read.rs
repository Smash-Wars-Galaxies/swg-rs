use std::fs::File;
use std::path::PathBuf;

use swg_stf::error::Result;
use swg_stf::read::StringTableReader;
use tracing_test::traced_test;
use widestring::u16cstr;

#[traced_test]
#[test]
fn parse_stf() -> Result<()> {
    // Create a path to the desired file
    let path = PathBuf::from(format!(
        "{}/resources/single_entry.stf",
        env!("CARGO_MANIFEST_DIR")
    ));

    let mut file = File::open(&path)?;
    let stf = StringTableReader::new(&mut file)?;

    assert_eq!(stf.len(), 1);

    let first_entry = stf.by_id("test");
    assert!(first_entry.is_some());

    assert_eq!(first_entry.unwrap(), u16cstr!("testing"));

    Ok(())
}
