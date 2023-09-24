use anyhow::Result;
use file_manager::aws_config::AwsConfig;
use file_manager::aws_credential::AwsCredential;
use shared::args::Args;
use std::collections::HashMap;

pub async fn login_profiles(
    configs: &HashMap<String, AwsConfig>,
    credentials: &mut HashMap<String, AwsCredential>,
    force: bool,
    args: &Args,
) -> Result<()> {
    for config in configs
        .iter()
        .filter(|(_, v)| v.credential_process.is_none())
    {
        let profile_name = config.0;
        let _ = login_internal(configs, credentials, profile_name, force, false, args).await?;
    }

    AwsCredential::write(&credentials)?;

    Ok(())
}

pub async fn login_profile(
    configs: &HashMap<String, AwsConfig>,
    credentials: &mut HashMap<String, AwsCredential>,
    profile_name: &str,
    force: bool,
    args: &Args,
) -> Result<AwsCredential> {
    let credential = login_internal(configs, credentials, &profile_name, force, true, args).await?;

    AwsCredential::write(&credentials)?;

    Ok(credential)
}

async fn login_internal(
    configs: &HashMap<String, AwsConfig>,
    credentials: &mut HashMap<String, AwsCredential>,
    profile_name: &str,
    force: bool,
    no_prompt: bool,
    args: &Args,
) -> Result<AwsCredential> {
    let credential =
        sso::sso::login(configs, credentials, profile_name, force, no_prompt, args).await?;

    AwsCredential::upsert(profile_name, &credential, credentials)?;

    Ok(credential)
}
