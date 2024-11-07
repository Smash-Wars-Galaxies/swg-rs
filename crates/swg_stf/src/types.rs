use derive_more::derive::{Constructor, Deref};
use std::collections::HashMap;
use widestring::U16String;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, PartialEq, Eq, Deref)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(from = "HashMap<String, String>", into = "HashMap<String, String>")
)]
pub struct StringTable(HashMap<String, U16String>);

#[cfg(feature = "serde")]
impl From<HashMap<String, String>> for StringTable {
    fn from(value: HashMap<String, String>) -> Self {
        Self::new(value.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

#[cfg(feature = "serde")]
impl From<StringTable> for HashMap<String, String> {
    fn from(value: StringTable) -> Self {
        value
            .0
            .into_iter()
            .map(|(k, v)| (k, v.to_string_lossy()))
            .collect()
    }
}
