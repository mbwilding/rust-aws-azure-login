use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use directories::UserDirs;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct AwsCredentials {
    pub version: u8,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    #[serde(serialize_with = "serialize_datetime_with_ms")]
    pub expiration: Option<DateTime<Utc>>,
}

fn serialize_datetime_with_ms<S>(
    dt: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dt {
        Some(actual_dt) => {
            let str_dt = actual_dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
            serializer.serialize_str(&str_dt)
        }
        None => serializer.serialize_none(),
    }
}

impl Default for AwsCredentials {
    fn default() -> Self {
        Self {
            version: 1,
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            expiration: None,
        }
    }
}

impl AwsCredentials {
    pub fn profile(profile_name: &str) -> Result<Self> {
        let config = Self::read_config()?;

        let aws_profile = config.get(profile_name).ok_or_else(|| {
            anyhow!(
                "Profile '{}' not found in the AWS credentials file",
                profile_name
            )
        })?;

        Self::from_ini_section(aws_profile)
    }

    pub fn profiles() -> Result<HashMap<String, Self>> {
        let config = Self::read_config()?;

        let profile_map: HashMap<String, Self> = config
            .iter()
            .filter_map(|(profile_name, section)| {
                Self::from_ini_section(section)
                    .map(|profile| (profile_name.clone(), profile))
                    .ok()
            })
            .collect();

        Ok(profile_map)
    }

    pub fn profile_default() -> Result<Self> {
        std::env::var("AWS_PROFILE")
            .map(|profile_name| Self::profile(&profile_name))
            .unwrap_or_else(|_| Self::profile("default"))
    }

    fn get_credentials_path() -> Result<String> {
        match UserDirs::new() {
            Some(user_dirs) => {
                let credentials_path = user_dirs.home_dir().join(".aws/credentials");
                if credentials_path.exists() {
                    credentials_path
                        .to_str()
                        .map(|s| s.to_owned())
                        .ok_or_else(|| anyhow!("Path contains invalid Unicode"))
                } else {
                    Err(anyhow!("AWS credentials file not found"))
                }
            }
            None => Err(anyhow!("Unable to get user directories")),
        }
    }

    fn read_config() -> Result<HashMap<String, HashMap<String, Option<String>>>> {
        let credentials_path = Self::get_credentials_path()?;
        let config = ini::macro_safe_load(&credentials_path)
            .map_err(|e| anyhow!("Failed to load INI: {:?}", e))?;

        Ok(config)
    }

    fn from_ini_section(section: &HashMap<String, Option<String>>) -> Result<AwsCredentials> {
        let access_key_id = section
            .get("aws_access_key_id")
            .and_then(|v| v.as_ref().cloned());

        let secret_access_key = section
            .get("aws_secret_access_key")
            .and_then(|v| v.as_ref().cloned());

        let session_token = section
            .get("aws_session_token")
            .and_then(|v| v.as_ref())
            .map(|s| s.trim_matches('"').to_string());

        let expiration = section
            .get("aws_expiration")
            .and_then(|v| v.as_ref())
            .map(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| anyhow!("Failed to parse datetime: {:?}", e))?;

        Ok(AwsCredentials {
            access_key_id,
            secret_access_key,
            session_token,
            expiration,
            ..Default::default()
        })
    }
}
