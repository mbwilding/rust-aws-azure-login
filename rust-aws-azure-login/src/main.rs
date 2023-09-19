use anyhow::Result;
use aws::aws_credentials::AwsCredentials;
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

    let profile = if args.profile.is_some() {
        args.profile.unwrap()
    } else {
        std::env::var("AWS_PROFILE").unwrap_or("default".to_string())
    };

    if args.configure {
        aws::aws_config::AwsConfig::configure_profile(&profile)?;
    } else {
        if args.all_profiles {
            println!("All profiles");
            let aws_credentials = web::login::login_all(args.force_refresh, args.no_prompt).await?;
            println!("{:?}", aws_credentials) // TODO: Testing only
        } else {
            println!("Profile: {}", profile);
            let aws_credential =
                web::login::login(&profile, args.force_refresh, args.no_prompt).await?;
            println!("{:?}", aws_credential) // TODO: Testing only
        }
    }

    Ok(())
}
