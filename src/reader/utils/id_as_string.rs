/// Helper to (de)serialize a `db::Id` as a string.
///
/// Useful as the API requires stream IDs to be strings.
/// Deserialization supports both decimal and hex notation,
/// so that this can be used for item IDs.
///
/// Usage: `#[serde(with = "id_as_string")]`
use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};
use std::fmt;

use crate::prelude::*;

pub fn serialize<S: Serializer>(id: &db::Id, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&id.inner().to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<db::Id, D::Error>
where
    D: Deserializer<'de>,
{
    struct Vis;

    impl<'de> Visitor<'de> for Vis {
        type Value = db::Id;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_str(Vis)
}
