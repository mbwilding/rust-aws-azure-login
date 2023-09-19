use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The name of the profile to log in with (or configure)
    #[arg(short, long, default_value = "")]
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

    /// Enables verbose logging to the console
    #[arg(short, long, default_value_t = true)] // TODO: default_value_t = false
    verbose: bool,
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

    let profile = resolve_profile_name(&args);
    println!("Profile: {}", profile);

    if args.configure {
        aws::aws_config::AwsConfig::configure_profile(&profile)?;
    } else {
        if args.all_profiles {
            web::login::login_all(args.force_refresh, args.no_prompt).await?;
        } else {
            web::login::login(&profile, args.force_refresh, args.no_prompt).await?;
        }
    }

    Ok(())
}

fn resolve_profile_name(args: &Args) -> String {
    if !args.profile.is_empty() {
        args.profile.clone()
    } else {
        std::env::var("AWS_PROFILE").unwrap_or("default".to_string())
    }
}
