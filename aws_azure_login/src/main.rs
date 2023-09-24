use clap::Parser;
use file_manager::aws_config::AwsConfig;
use file_manager::aws_credentials::AwsCredentials;

mod config;
mod login;

/// Required due to using the stderr writer vs no writer specified
/// SubscriberBuilder<fn() -> Stderr> vs SubscriberBuilder
#[macro_export]
macro_rules! init_logging {
    ($builder:expr, $debug:expr) => {
        let logging = $builder;

        let level = if $debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        };

        logging
            .with_max_level(level)
            .with_target($debug)
            .with_line_number($debug)
            .init();
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = shared::args::Args::parse();

    if args.json {
        let logging = tracing_subscriber::fmt().with_writer(std::io::stderr);
        init_logging!(logging, args.debug);
    } else {
        let logging = tracing_subscriber::fmt();
        init_logging!(logging, args.debug);
    }

    let profile_name = args
        .profile
        .clone()
        .unwrap_or_else(|| std::env::var("AWS_PROFILE").unwrap_or("default".to_string()));

    if args.configure {
        let mut configs = AwsConfig::read_file().unwrap_or_default();
        config::configure_profile(&mut configs, &profile_name)?;
        return Ok(());
    }

    let configs = AwsConfig::read_file()?;
    let mut credentials = AwsCredentials::read_file().unwrap_or_default();

    if args.all {
        login::login_profiles(&configs, &mut credentials, args.force, &args).await?;
    } else {
        login::login_profile(&configs, &mut credentials, &profile_name, args.force, &args).await?;
    }

    Ok(())
}
