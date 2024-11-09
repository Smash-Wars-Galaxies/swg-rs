use std::path::PathBuf;

use swg_iff::error::Error;
// use swg_iff::iff::{GroupChunk};

#[test]
fn parse_iff() -> Result<(), Error> {
    // Create a path to the desired file
    let path = PathBuf::from(format!(
        "{}/resources/skills.iff",
        env!("CARGO_MANIFEST_DIR")
    ));

    let mut buffer = std::fs::read(path)?;
    // let iff = GroupChunk::parse(&buffer)?;
    // println!("{:#?}", iff);

    assert!(false);
    Ok(())
}

// #[test]
// fn parse_datatable() -> Result<(), Error> {
//     // Create a path to the desired file
//     let path = PathBuf::from(format!(
//         "{}/resources/skills.iff",
//         env!("CARGO_MANIFEST_DIR")
//     ));

//     let mut file = File::open(&path)?;
//     let table = DataTable::try_from(IFFFile::read_be(&mut file)?)?;

//     assert_eq!(table.columns.len(), 27);
//     assert_eq!(table.types.len(), table.columns.len());

//     assert_eq!(table.rows.len(), 1067);

//     let row = &table.rows[14];
//     assert_eq!(row.cells.len(), table.columns.len());

//     assert_eq!(row.cells[0].name, "NAME".into());
//     assert_eq!(
//         row.cells[0].data,
//         CellData::String(NullString("social_entertainer_hairstyle_02".into()))
//     );

//     assert_eq!(row.cells[2].name, "GRAPH_TYPE".into());
//     assert_eq!(row.cells[2].data, CellData::Enum(4));

//     assert_eq!(row.cells[7].name, "MONEY_REQUIRED".into());
//     assert_eq!(row.cells[7].data, CellData::Integer(2000));

//     assert_eq!(row.cells[8].name, "POINTS_REQUIRED".into());
//     assert_eq!(row.cells[8].data, CellData::Integer(3));

//     Ok(())
// }
