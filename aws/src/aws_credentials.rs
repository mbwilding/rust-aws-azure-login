use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use directories::UserDirs;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct AwsCredentials {
    #[serde(skip)]
    pub profile_name: Option<String>,
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub aws_session_token: Option<String>,
    #[serde(serialize_with = "serialize_datetime_with_ms")]
    pub aws_expiration: Option<DateTime<Utc>>,
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

impl AwsCredentials {
    pub fn profile(profile_name: &str) -> Result<Self> {
        let config = Self::read_config()?;

        let aws_profile = config.get(profile_name).ok_or_else(|| {
            anyhow!(
                "Profile '{}' not found in the AWS credentials file",
                profile_name
            )
        })?;

        Self::from_ini_section(profile_name, aws_profile)
    }

    pub fn profiles() -> Result<HashMap<String, Self>> {
        let config = Self::read_config()?;

        let profile_map: HashMap<String, Self> = config
            .iter()
            .filter_map(|(profile_name, section)| {
                Self::from_ini_section(profile_name, section)
                    .map(|profile| (profile_name.clone(), profile))
                    .ok()
            })
            .collect();

        Ok(profile_map)
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

    fn from_ini_section(
        profile_name: &str,
        section: &HashMap<String, Option<String>>,
    ) -> Result<AwsCredentials> {
        let aws_access_key_id = section
            .get("aws_access_key_id")
            .and_then(|v| v.as_ref().cloned());

        let aws_secret_access_key = section
            .get("aws_secret_access_key")
            .and_then(|v| v.as_ref().cloned());

        let aws_session_token = section
            .get("aws_session_token")
            .and_then(|v| v.as_ref())
            .map(|s| s.trim_matches('"').to_string());

        let aws_expiration = section
            .get("aws_expiration")
            .and_then(|v| v.as_ref())
            .map(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc)))
            .transpose()
            .map_err(|e| anyhow!("Failed to parse datetime: {:?}", e))?;

        Ok(AwsCredentials {
            profile_name: Some(profile_name.to_owned()),
            aws_access_key_id,
            aws_secret_access_key,
            aws_session_token,
            aws_expiration,
        })
    }

    pub fn is_profile_about_to_expire(&self) -> bool {
        match self.aws_expiration {
            Some(expiration_date) => {
                let time_difference = expiration_date.signed_duration_since(Utc::now());
                time_difference < chrono::Duration::minutes(11)
            }
            None => true,
        }
    }

    pub fn set_profile_credentials(profile_name: &str, profile: AwsCredentials) -> Result<()> {
        let aws_expiration = profile
            .aws_expiration
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();

        // TODO: Find a library that can write INI files

        Ok(())
    }
}
