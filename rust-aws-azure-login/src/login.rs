use anyhow::Result;
use aws::aws_config::AwsConfig;
use aws::aws_credentials::AwsCredentials;
use std::collections::HashMap;

pub async fn login_profiles(force_refresh: bool) -> Result<()> {
    let config = AwsConfig::read_config()?;
    let mut credentials = AwsCredentials::read_config()?;

    for profile_config in config.iter() {
        let profile_name = profile_config.0;
        login_internal(profile_name, force_refresh, true, &mut credentials).await?;
    }

    AwsCredentials::write_config(&credentials)?;

    Ok(())
}

pub async fn login_profile(profile_name: &str, force_refresh: bool, no_prompt: bool) -> Result<()> {
    let mut credentials = AwsCredentials::read_config()?;

    let profile_name = if profile_name != "default" {
        format!("profile {}", profile_name)
    } else {
        profile_name.to_string()
    };

    login_internal(&profile_name, force_refresh, no_prompt, &mut credentials).await?;

    AwsCredentials::write_config(&credentials)?;

    Ok(())
}

async fn login_internal(
    profile_name: &str,
    force_refresh: bool,
    no_prompt: bool,
    aws_credentials: &mut HashMap<String, AwsCredentials>,
) -> Result<()> {
    let profile_credentials = web::login::login(profile_name, force_refresh, no_prompt).await?;
    let _ = aws_credentials.insert(profile_name.to_string(), profile_credentials);

    Ok(())
}
