use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use file_manager::aws_config::AwsConfig;
use std::collections::HashMap;
use tracing::log::info;

pub fn configure_profile(
    profiles: &mut HashMap<String, AwsConfig>,
    profile_name: &str,
) -> Result<()> {
    let profile = AwsConfig::get(profile_name, profiles).unwrap_or_default();

    info!("Configuring profile: {}", profile_name);

    let azure_tenant_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Tenant ID")
        .with_initial_text(profile.azure_tenant_id.unwrap_or_default())
        .allow_empty(false)
        .interact_text()
        .unwrap();

    let azure_app_id_uri: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure App ID URI")
        .with_initial_text(profile.azure_app_id_uri.unwrap_or_default())
        .allow_empty(false)
        .interact_text()
        .unwrap();

    let azure_default_username: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Username")
        .default(profile.azure_default_username.unwrap_or_default())
        .allow_empty(true)
        .interact_text()
        .unwrap();

    let azure_default_password: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Password")
        .default(profile.azure_default_password.unwrap_or_default())
        .allow_empty(true)
        .interact_text()
        .unwrap();

    let azure_default_role_arn: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Default Role ARN (if multiple)")
        .default(profile.azure_default_role_arn.unwrap_or_default())
        .allow_empty(true)
        .interact_text()
        .unwrap();

    let azure_default_duration_hours: u8 = loop {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Default Session Duration Hours (up to 12)")
            .default(
                profile
                    .azure_default_duration_hours
                    .unwrap_or_default()
                    .to_string(),
            )
            .allow_empty(false)
            .interact_text()
            .unwrap();

        if let Ok(value) = input.parse::<u8>() {
            if value > 0 && value <= 12 {
                break value;
            }
        }
    };

    let azure_default_remember_me: bool = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Remember Me")
        .default(profile.azure_default_remember_me.unwrap_or_default())
        .allow_empty(false)
        .interact_text()
        .unwrap();

    let region: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Region")
        .default(profile.region.unwrap_or_default())
        .allow_empty(false)
        .interact_text()
        .unwrap()
        .into();

    let okta_default_username: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Okta Username")
        .default(profile.okta_default_username.unwrap_or_default())
        .allow_empty(true)
        .interact_text()
        .unwrap();

    let okta_default_password: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Okta Password")
        .default(profile.okta_default_password.unwrap_or_default())
        .allow_empty(true)
        .interact_text()
        .unwrap();

    let new_profile = AwsConfig {
        azure_tenant_id: if azure_tenant_id.trim().is_empty() {
            None
        } else {
            Some(azure_tenant_id)
        },
        azure_app_id_uri: if azure_app_id_uri.trim().is_empty() {
            None
        } else {
            Some(azure_app_id_uri)
        },
        azure_default_username: if azure_default_username.trim().is_empty() {
            None
        } else {
            Some(azure_default_username)
        },
        azure_default_password: if azure_default_password.trim().is_empty() {
            None
        } else {
            Some(azure_default_password)
        },
        azure_default_role_arn: if azure_default_role_arn.trim().is_empty() {
            None
        } else {
            Some(azure_default_role_arn)
        },
        azure_default_duration_hours: Some(azure_default_duration_hours),
        azure_default_remember_me: Some(azure_default_remember_me),
        region: if region.trim().is_empty() {
            None
        } else {
            Some(region)
        },
        okta_default_username: if okta_default_username.trim().is_empty() {
            None
        } else {
            Some(okta_default_username)
        },
        okta_default_password: if okta_default_password.trim().is_empty() {
            None
        } else {
            Some(okta_default_password)
        },
        credential_process: None,
    };

    AwsConfig::upsert(profile_name, &new_profile, profiles)?;
    AwsConfig::write(&profiles)?;

    Ok(())
}
