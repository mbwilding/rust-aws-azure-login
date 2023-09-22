use crate::serde_support::{
    deserialize_string_to_bool, serialize_bool_to_string, serialize_ordered,
};
use anyhow::{anyhow, Result};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
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

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            azure_tenant_id: None,
            azure_app_id_uri: Some("https://signin.aws.amazon.com/saml".to_string()),
            azure_default_username: None,
            azure_default_password: None,
            azure_default_role_arn: None,
            azure_default_duration_hours: Some(8),
            azure_default_remember_me: Some(true),
            region: Some("ap-southeast-2".to_string()),
            okta_default_username: None,
            okta_default_password: None,
            credential_process: None,
        }
    }
}

impl AwsConfig {
    fn file_path() -> Result<PathBuf> {
        match UserDirs::new() {
            Some(user_dirs) => {
                let config_path = user_dirs.home_dir().join(".aws/config");
                if config_path.exists() {
                    Ok(config_path)
                } else {
                    Err(anyhow!(
                        "AWS config file not found, please run with -c or --configure"
                    ))
                }
            }
            None => Err(anyhow!("Unable to get user directories")),
        }
    }

    pub fn read_file() -> Result<HashMap<String, AwsConfig>> {
        let credentials_path = Self::file_path()?;
        let file = File::open(credentials_path)?;
        let reader = BufReader::new(file);
        let aws_credentials: HashMap<String, AwsConfig> = serde_ini::from_bufread(reader)?;

        Ok(aws_credentials)
    }

    pub fn write(profiles: &HashMap<String, AwsConfig>) -> Result<()> {
        let credentials_path = Self::file_path()?;
        serialize_ordered(profiles, credentials_path)
    }

    pub fn get(profile_name: &str, profiles: &HashMap<String, AwsConfig>) -> Result<AwsConfig> {
        let profile_name_sanitized = Self::sanitize_profile_name(profile_name);
        let profile = profiles.get(&profile_name_sanitized).ok_or_else(|| {
            anyhow!(
                "Profile '{}' not found in the AWS config file, please run with -c or --configure",
                profile_name
            )
        })?;

        Ok(profile.clone())
    }

    pub fn upsert(
        profile_name: &str,
        profile: &AwsConfig,
        profiles: &mut HashMap<String, AwsConfig>,
    ) -> Result<()> {
        let profile_name_sanitized = Self::sanitize_profile_name(profile_name);

        let _ = profiles.insert(profile_name_sanitized, profile.to_owned());

        Ok(())
    }

    pub fn sanitize_profile_name(profile_name: &str) -> String {
        if profile_name != "default" && !profile_name.starts_with("profile ") {
            format!("profile {}", profile_name)
        } else {
            profile_name.to_string()
        }
    }
}
