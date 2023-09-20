use anyhow::{anyhow, Result};
use directories::UserDirs;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AwsConfig {
    // TODO: Possibly make a hash map
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure_tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure_app_id_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure_default_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure_default_password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure_default_role_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure_default_duration_hours: Option<u8>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_bool_to_string",
        deserialize_with = "deserialize_string_to_bool"
    )]
    pub azure_default_remember_me: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_default_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okta_default_password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_process: Option<String>,
}

fn serialize_bool_to_string<S>(value: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(true) => serializer.serialize_str("true"),
        Some(false) => serializer.serialize_str("false"),
        None => serializer.serialize_none(),
    }
}

fn deserialize_string_to_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
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

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            azure_tenant_id: None,
            azure_app_id_uri: None,
            azure_default_username: None,
            azure_default_password: None,
            azure_default_role_arn: None,
            azure_default_duration_hours: None,
            azure_default_remember_me: None,
            region: None,
            okta_default_username: None,
            okta_default_password: None,
            credential_process: None,
        }
    }
}

impl AwsConfig {
    fn get_config_path() -> Result<PathBuf> {
        match UserDirs::new() {
            Some(user_dirs) => {
                let config_path = user_dirs.home_dir().join(".aws/config");
                if config_path.exists() {
                    Ok(config_path)
                } else {
                    Err(anyhow!("AWS config file not found"))
                }
            }
            None => Err(anyhow!("Unable to get user directories")),
        }
    }

    pub fn read_config() -> Result<HashMap<String, AwsConfig>> {
        let credentials_path = Self::get_config_path()?;
        let file = File::open(credentials_path)?;
        let reader = BufReader::new(file);
        let aws_credentials: HashMap<String, AwsConfig> = serde_ini::from_bufread(reader)?;

        Ok(aws_credentials)
    }

    pub fn write_config(profiles: &HashMap<String, AwsConfig>) -> Result<()> {
        let credentials_path = Self::get_config_path()?;
        let file = File::create(credentials_path)?;
        let writer = BufWriter::new(file);
        serde_ini::to_writer(writer, profiles)?;

        Ok(())
    }
}
