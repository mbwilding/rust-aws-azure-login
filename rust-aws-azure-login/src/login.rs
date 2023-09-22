use anyhow::Result;
use aws::aws_config::AwsConfig;
use aws::aws_credentials::AwsCredentials;
use shared::args::Args;
use std::collections::HashMap;

pub async fn login_profiles(
    configs: &HashMap<String, AwsConfig>,
    credentials: &mut HashMap<String, AwsCredentials>,
    force_refresh: bool,
    args: &Args,
) -> Result<()> {
    for config in configs
        .iter()
        .filter(|(_, v)| v.credential_process.is_none())
    {
        let profile_name = config.0;
        login_internal(
            configs,
            credentials,
            profile_name,
            force_refresh,
            true,
            args,
        )
        .await?;
    }

    AwsCredentials::write(&credentials)?;

    Ok(())
}

pub async fn login_profile(
    configs: &HashMap<String, AwsConfig>,
    credentials: &mut HashMap<String, AwsCredentials>,
    profile_name: &str,
    force_refresh: bool,
    no_prompt: bool,
    args: &Args,
) -> Result<()> {
    login_internal(
        configs,
        credentials,
        &profile_name,
        force_refresh,
        no_prompt,
        args,
    )
    .await?;

    AwsCredentials::write(&credentials)?;

    Ok(())
}

async fn login_internal(
    configs: &HashMap<String, AwsConfig>,
    credentials: &mut HashMap<String, AwsCredentials>,
    profile_name: &str,
    force_refresh: bool,
    no_prompt: bool,
    args: &Args,
) -> Result<()> {
    let profile_credentials = web::login::login(
        configs,
        credentials,
        profile_name,
        force_refresh,
        no_prompt,
        args,
    )
    .await?;
    AwsCredentials::upsert(profile_name, &profile_credentials, credentials)?;

    Ok(())
}
