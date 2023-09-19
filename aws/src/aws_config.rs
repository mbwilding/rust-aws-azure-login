use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use directories::UserDirs;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AwsConfig {
    pub azure_tenant_id: Option<String>,
    pub azure_app_id_uri: Option<String>,
    pub azure_default_username: Option<String>,
    pub azure_default_password: Option<String>,
    pub azure_default_role_arn: Option<String>,
    pub azure_default_duration_hours: Option<u8>,
    pub region: Option<String>,
    pub azure_default_remember_me: Option<bool>,
    pub okta_default_username: Option<String>,
    pub okta_default_password: Option<String>,
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
            .map_err(|e| anyhow!("Failed to open INI: {:?}", e))?;

        Ok(config)
    }

    fn from_ini_section(section: &HashMap<String, Option<String>>) -> Result<Self> {
        let azure_tenant_id = section.get("azure_tenant_id").cloned().unwrap_or(None);

        let azure_app_id_uri = section.get("azure_app_id_uri").cloned().unwrap_or(None);

        let azure_default_username = section
            .get("azure_default_username")
            .cloned()
            .unwrap_or(None);

        let azure_default_role_arn = section
            .get("azure_default_role_arn")
            .cloned()
            .unwrap_or(None);

        let azure_default_duration_hours = section
            .get("azure_default_duration_hours")
            .and_then(|v| v.clone())
            .and_then(|s| s.parse().ok());

        let region = section.get("region").cloned().unwrap_or(None);

        let azure_default_remember_me = section
            .get("azure_default_remember_me")
            .and_then(|v| v.clone())
            .and_then(|s| s.parse().ok());

        let credential_process = section.get("credential_process").cloned().unwrap_or(None);

        Ok(Self {
            azure_tenant_id,
            azure_app_id_uri,
            azure_default_username,
            azure_default_password: None,
            azure_default_role_arn,
            azure_default_duration_hours,
            region,
            azure_default_remember_me,
            okta_default_username: None,
            okta_default_password: None,
            credential_process,
        })
    }

    pub fn configure_profile(profile_name: &str) -> Result<()> {
        let mut profile = AwsConfig::profile(profile_name)?;

        let azure_tenant_id: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Azure Tenant ID")
            .with_initial_text(profile.azure_tenant_id.unwrap_or_default())
            .interact_text()
            .unwrap();

        let azure_app_id_uri: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Azure App ID URI")
            .with_initial_text(profile.azure_app_id_uri.unwrap_or_default())
            .interact_text()
            .unwrap();

        let azure_default_username: Option<String> = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Azure Username")
            .default(profile.azure_default_username.unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .unwrap()
            .into();

        let azure_default_password: Option<String> = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Azure Password")
            .default(profile.azure_default_password.unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .unwrap()
            .into();

        let azure_default_role_arn: Option<String> = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Default Role ARN (if multiple)")
            .default(profile.azure_default_role_arn.unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .unwrap()
            .into();

        let azure_default_duration_hours: Option<u8> = loop {
            let input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Default Session Duration Hours (up to 12)")
                .default(
                    profile
                        .azure_default_duration_hours
                        .unwrap_or_default()
                        .to_string(),
                )
                .allow_empty(true)
                .interact_text()
                .unwrap();

            if let Ok(value) = input.parse::<u8>() {
                if value > 0 && value <= 12 {
                    break Some(value);
                }
            }
        };

        let region: Option<String> = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Region")
            .default(profile.region.unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .unwrap()
            .into();

        let azure_default_remember_me: Option<bool> =
            Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Remember Me")
                .default(profile.azure_default_remember_me.unwrap_or_default())
                .interact()
                .unwrap()
                .into();

        let okta_default_username: Option<String> = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Okta Username")
            .default(profile.okta_default_username.unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .unwrap()
            .into();

        let okta_default_password: Option<String> = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Okta Password")
            .default(profile.okta_default_password.unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .unwrap()
            .into();

        profile = Self {
            azure_tenant_id: Some(azure_tenant_id),
            azure_app_id_uri: Some(azure_app_id_uri),
            azure_default_username,
            azure_default_password,
            azure_default_role_arn,
            azure_default_duration_hours,
            region,
            azure_default_remember_me,
            okta_default_username,
            okta_default_password,
            credential_process: None,
        };

        Self::set_profile_config(profile_name, profile)?;

        Ok(())
    }

    pub fn set_profile_config(profile_name: &str, profile: AwsConfig) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let mut config = ini::macro_safe_load(&config_path)
            .map_err(|e| anyhow!("Failed to open INI: {:?}", e))?;

        let profile_name = if profile_name != "default" {
            format!("profile {}", profile_name)
        } else {
            profile_name.to_string()
        };

        // TODO: Find a library that can write INI files

        Ok(())
    }
}
