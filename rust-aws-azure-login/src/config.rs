use anyhow::Result;
use aws::aws_config::AwsConfig;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;

pub fn configure_profile(profile_name: &str) -> Result<()> {
    let mut config = AwsConfig::read_config()?;
    let profile = config
        .get(profile_name)
        .ok_or_else(AwsConfig::default)
        .unwrap()
        .clone();

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

    let azure_default_remember_me: Option<bool> = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Remember Me")
        .default(profile.azure_default_remember_me.unwrap_or_default())
        .allow_empty(true)
        .interact_text()
        .unwrap()
        .into();

    let region: Option<String> = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Region")
        .default(profile.region.unwrap_or_default())
        .allow_empty(true)
        .interact_text()
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

    let new_profile = AwsConfig {
        azure_tenant_id: Some(azure_tenant_id),
        azure_app_id_uri: Some(azure_app_id_uri),
        azure_default_username,
        azure_default_password,
        azure_default_role_arn,
        azure_default_duration_hours,
        azure_default_remember_me,
        region,
        okta_default_username,
        okta_default_password,
        credential_process: None,
    };

    config.insert(profile_name.to_owned(), new_profile);
    AwsConfig::write_config(&config)?;

    Ok(())
}
