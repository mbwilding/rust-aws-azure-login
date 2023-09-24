use clap::Parser;
use file_manager::aws_config::AwsConfig;
use file_manager::aws_credentials::AwsCredentials;

mod config;
mod login;

#[macro_export]
macro_rules! init_logging {
    ($builder:expr, $debug:expr) => {
        let logging = $builder;

        if $debug {
            logging
                .with_max_level(tracing::Level::DEBUG)
                .with_target(true)
                .with_line_number(true)
                .init();
        } else {
            logging
                .with_max_level(tracing::Level::INFO)
                .with_target(false)
                .with_line_number(false)
                .init();
        }
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = shared::args::Args::parse();

    // TODO: Refactor
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
