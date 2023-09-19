mod config;
mod login;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The name of the profile to log in with (or configure)
    #[arg(short, long)]
    profile: Option<String>,

    /// Run for all configured profiles
    #[arg(short, long, default_value_t = false)]
    all_profiles: bool,

    /// Force a credential refresh, even if they are still valid
    #[arg(short, long, default_value_t = false)]
    force_refresh: bool,

    /// Configure the profile
    #[arg(short, long, default_value_t = false)]
    configure: bool,

    /// 'cli' hides the login page and perform the login through the CLI;
    /// 'gui' performs the login through the Azure GUI;
    /// 'debug' shows the login page but perform the login through the CLI
    #[arg(short, long, default_value = "cli")]
    mode: String, // TODO: implement this

    /// Do not prompt for input and accept the default choice
    #[arg(short, long, default_value_t = false)]
    no_prompt: bool,

    /// Enables verbose logging to the console
    #[arg(short, long, default_value_t = cfg!(debug_assertions))]
    verbose: bool,

    /// Additionally returns the JSON credentials to stdout, for consumption by AWS Config [credential_process]
    #[arg(short, long, default_value_t = false)]
    json: bool, // TODO: implement this
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true)
            .with_line_number(true)
            .init();
    }

    let profile_name = if args.profile.is_some() {
        args.profile.unwrap()
    } else {
        std::env::var("AWS_PROFILE").unwrap_or("default".to_string())
    };

    if args.configure {
        config::configure_profile(&profile_name)?;
    } else if args.all_profiles {
        login::login_profiles(args.force_refresh).await?;
    } else {
        login::login_profile(&profile_name, args.force_refresh, args.no_prompt).await?;
    }

    Ok(())
}
