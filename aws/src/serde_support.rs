use serde::{de, Deserialize, Deserializer, Serializer};

pub fn serialize_bool_to_string<S>(value: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    match value {
        Some(true) => serializer.serialize_str("true"),
        Some(false) => serializer.serialize_str("false"),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_string_to_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    match s {
        Some(ref value) if value == "true" => Ok(Some(true)),
        Some(ref value) if value == "false" => Ok(Some(false)),
        Some(_) => Err(de::Error::custom("expected true or false as string")),
        None => Ok(None),
    }
}