use aws::aws_config::AwsConfig;
use aws::aws_credentials::AwsCredentials;
use clap::Parser;

mod config;
mod login;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = shared::args::Args::parse();

    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true)
            .with_line_number(true)
            .init();
    }

    let profile_name = args
        .profile
        .clone()
        .unwrap_or_else(|| std::env::var("AWS_PROFILE").unwrap_or("default".to_string()));

    if args.configure {
        let mut configs = AwsConfig::read_file()?;
        config::configure_profile(&mut configs, &profile_name)?;
    }

    let configs = AwsConfig::read_file().unwrap_or_default();
    let mut credentials = AwsCredentials::read_file().unwrap_or_default();

    if args.all {
        login::login_profiles(&configs, &mut credentials, args.force_refresh, &args).await?;
    } else {
        login::login_profile(
            &configs,
            &mut credentials,
            &profile_name,
            args.force_refresh,
            args.no_prompt,
            &args,
        )
        .await?;
    }

    Ok(())
}
