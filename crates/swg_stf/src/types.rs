use derive_more::derive::{Constructor, Deref};
use std::collections::HashMap;
use widestring::U16String;

#[cfg(feature = "polars")]
use polars::{error::PolarsError, frame::DataFrame, prelude::*};

#[derive(Constructor, Clone, Debug, PartialEq, Eq, Deref)]
pub struct StringTable(HashMap<String, U16String>);

#[cfg(feature = "polars")]
impl TryFrom<StringTable> for DataFrame {
    type Error = PolarsError;

    fn try_from(value: StringTable) -> Result<Self, Self::Error> {
        let mut keys = Vec::new();
        let mut values = Vec::new();

        for (k, v) in value.0.into_iter() {
            keys.push(k);
            values.push(
                v.to_string()
                    .map_err(|e| polars_err!(ComputeError: "invalid utf8: {}", e))?,
            );
        }

        DataFrame::new(vec![
            Column::new("id".into(), keys),
            Column::new("value".into(), values),
        ])
    }
}
