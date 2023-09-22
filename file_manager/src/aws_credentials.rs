use crate::serde_support::serialize_ordered;
use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Utc};
use directories::UserDirs;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AwsCredentials {
    // TODO: Possibly make a hash map
    #[serde(skip)]
    pub profile_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_access_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_secret_access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_session_token: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_datetime_with_ms"
    )]
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
    fn file_path() -> Result<PathBuf> {
        match UserDirs::new() {
            Some(user_dirs) => Ok(user_dirs.home_dir().join(".aws/credentials")),
            None => Err(anyhow!("Unable to get user directories")),
        }
    }

    pub fn read_file() -> Result<HashMap<String, AwsCredentials>> {
        let credentials_path = Self::file_path()?;
        if !credentials_path.exists() {
            bail!("AWS credentials file not found")
        }
        let file = File::open(credentials_path)?;
        let reader = BufReader::new(file);
        let aws_credentials: HashMap<String, AwsCredentials> = serde_ini::from_bufread(reader)?;

        Ok(aws_credentials)
    }

    pub fn write(profiles: &HashMap<String, AwsCredentials>) -> Result<()> {
        let credentials_path = Self::file_path()?;
        serialize_ordered(profiles, credentials_path)
    }

    pub fn get(
        profile_name: &str,
        profiles: &HashMap<String, AwsCredentials>,
    ) -> Result<AwsCredentials> {
        let profile = profiles.get(profile_name).ok_or_else(|| {
            anyhow!(
                "Profile '{}' not found in the AWS credentials file",
                profile_name
            )
        })?;

        Ok(profile.clone())
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

    pub fn upsert(
        profile_name: &str,
        profile: &AwsCredentials,
        profiles: &mut HashMap<String, AwsCredentials>,
    ) -> Result<()> {
        let _ = profiles.insert(profile_name.to_owned(), profile.to_owned());

        Ok(())
    }
}
