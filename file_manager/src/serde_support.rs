use anyhow::Result;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tracing::info;

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

pub enum FileName {
    Config,
    Credentials,
}

impl Display for FileName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileName::Config => write!(f, "Config"),
            FileName::Credentials => write!(f, "Credentials"),
        }
    }
}

pub fn serialize_write_ordered<T>(
    profiles: &HashMap<String, T>,
    path: PathBuf,
    file_name: FileName,
) -> Result<()>
where
    T: Serialize,
{
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let mut sorted_profiles: Vec<_> = profiles.iter().collect();
    sorted_profiles.sort_by_key(|x| x.0);

    for (key, profile) in sorted_profiles {
        writeln!(writer, "[{}]", key)?;
        serde_ini::to_writer(&mut writer, profile)?;
        writeln!(writer)?;
    }

    info!("AWS {} file modified", file_name);

    Ok(())
}
