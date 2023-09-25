use crate::json::JsonCredential;
use clap::Parser;
use file_manager::aws_config::AwsConfig;
use file_manager::aws_credential::AwsCredential;
use tracing_subscriber::EnvFilter;

mod config;
mod json;

/// Required due to using the stderr writer vs no writer specified
/// SubscriberBuilder<fn() -> Stderr> vs SubscriberBuilder
#[macro_export]
macro_rules! init_tracing {
    ($builder:expr, $debug:expr) => {
        let logging = $builder;

        logging
            .with_target($debug)
            .with_line_number($debug)
            .with_env_filter(EnvFilter::from("headless_chrome=off,tungstenite=off"))
            .init();
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = shared::args::Args::parse();

    if args.json {
        let logging = tracing_subscriber::fmt().with_writer(std::io::stderr);
        init_tracing!(logging, args.debug);
    } else {
        let logging = tracing_subscriber::fmt();
        init_tracing!(logging, args.debug);
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
    let mut credentials = AwsCredential::read_file().unwrap_or_default();

    if args.all {
        sso::sso::login_all(&configs, &mut credentials, &args).await?;
    } else {
        let credential = sso::sso::login(&configs, &mut credentials, &profile_name, &args).await?;

        if args.json {
            let json_credentials = JsonCredential::convert(credential);
            let json = serde_json::to_string_pretty(&json_credentials)?;
            println!("{}", json);
        }
    }

    Ok(())
}
