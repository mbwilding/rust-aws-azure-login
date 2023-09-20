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
        config::configure_profile(&profile_name)?;
    } else if args.all {
        login::login_profiles(args.force_refresh, &args).await?;
    } else {
        login::login_profile(&profile_name, args.force_refresh, args.no_prompt, &args).await?;
    }

    Ok(())
}
