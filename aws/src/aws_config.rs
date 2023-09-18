use anyhow::{anyhow, Result};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct AwsConfig {
    pub azure_tenant_id: Option<String>,
    pub azure_app_id_uri: Option<String>,
    pub azure_default_username: Option<String>,
    pub azure_default_role_arn: Option<String>,
    pub azure_default_duration_hours: Option<u8>,
    pub region: Option<String>,
    pub azure_default_remember_me: Option<bool>,
    pub credential_process: Option<String>,
}

impl AwsConfig {
    pub fn profile(profile_name: &str) -> Result<Self> {
        let config = Self::read_config()?;

        let prefixed_profile_name = if profile_name != "default" {
            format!("profile {}", profile_name)
        } else {
            profile_name.to_string()
        };

        let aws_profile = config
            .get(&prefixed_profile_name)
            .or_else(|| config.get(profile_name))
            .ok_or_else(|| {
                anyhow!(
                    "Profile '{}' not found in the AWS config file",
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
                let stripped_profile_name = profile_name
                    .strip_prefix("profile ")
                    .unwrap_or(profile_name);
                Self::from_ini_section(section)
                    .map(|profile| (stripped_profile_name.to_string(), profile))
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

    fn get_config_path() -> Result<String> {
        match UserDirs::new() {
            Some(user_dirs) => {
                let credentials_path = user_dirs.home_dir().join(".aws/config");
                if credentials_path.exists() {
                    credentials_path
                        .to_str()
                        .map(|s| s.to_owned())
                        .ok_or_else(|| anyhow!("Path contains invalid Unicode"))
                } else {
                    Err(anyhow!("AWS config file not found"))
                }
            }
            None => Err(anyhow!("Unable to get user directories")),
        }
    }

    fn read_config() -> Result<HashMap<String, HashMap<String, Option<String>>>> {
        let config_path = Self::get_config_path()?;
        let config = ini::macro_safe_load(&config_path)
            .map_err(|e| anyhow!("Failed to load INI: {:?}", e))?;

        Ok(config)
    }

    fn from_ini_section(section: &HashMap<String, Option<String>>) -> Result<Self> {
        let azure_tenant_id = section
            .get("azure_tenant_id")
            .cloned()
            .unwrap_or_else(|| None);

        let azure_app_id_uri = section
            .get("azure_app_id_uri")
            .cloned()
            .unwrap_or_else(|| None);

        let azure_default_username = section
            .get("azure_default_username")
            .cloned()
            .unwrap_or_else(|| None);

        let azure_default_role_arn = section
            .get("azure_default_role_arn")
            .cloned()
            .unwrap_or_else(|| None);

        let azure_default_duration_hours = section
            .get("azure_default_duration_hours")
            .and_then(|v| v.clone())
            .and_then(|s| s.parse().ok());

        let region = section.get("region").cloned().unwrap_or_else(|| None);

        let azure_default_remember_me = section
            .get("azure_default_remember_me")
            .and_then(|v| v.clone())
            .and_then(|s| s.parse().ok());

        let credential_process = section
            .get("credential_process")
            .cloned()
            .unwrap_or_else(|| None);

        Ok(Self {
            azure_tenant_id,
            azure_app_id_uri,
            azure_default_username,
            azure_default_role_arn,
            azure_default_duration_hours,
            region,
            azure_default_remember_me,
            credential_process,
        })
    }
}
