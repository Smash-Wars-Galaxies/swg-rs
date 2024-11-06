use std::{collections::HashMap, fmt};

use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};
use widestring::U16String;

use crate::types::StringTable;

#[cfg(feature = "serde")]
impl Serialize for StringTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            map.serialize_entry(k, &v.to_string_lossy())?;
        }
        map.end()
    }
}

struct StringTableVisitor {}

impl StringTableVisitor {
    fn new() -> Self {
        StringTableVisitor {}
    }
}

impl<'de> Visitor<'de> for StringTableVisitor {
    type Value = HashMap<String, U16String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string/string map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

        loop {
            let key = access.next_key()?;
            if key.is_none() {
                break;
            }

            let value = access.next_value::<String>()?;

            map.insert(key.unwrap(), U16String::from_str(&value));
        }

        Ok(map)
    }
}

impl<'de> Deserialize<'de> for StringTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(StringTable::new(
            deserializer.deserialize_map(StringTableVisitor::new())?,
        ))
    }
}
