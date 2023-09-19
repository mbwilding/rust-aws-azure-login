use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The name of the profile to log in with (or configure)
    #[arg(short, long, default_value = "default")]
    profile: String,

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
    mode: String,

    /// Do not prompt for input and accept the default choice
    #[arg(short, long, default_value_t = false)]
    no_prompt: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        //.json()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_line_number(true)
        .init();

    let args = Args::parse();

    if args.all_profiles {
        web::login::login_all(args.force_refresh, args.no_prompt).await?;
    }

    // TODO: For testing
    web::login::login("default", args.force_refresh, args.no_prompt).await?;

    Ok(())
}
