#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The name of the profile to log in with (or configure)
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Run for all configured profiles
    #[arg(short, long, default_value_t = false)]
    pub all: bool,

    /// Force a credential refresh, even if they are still valid
    #[arg(short, long, default_value_t = false)]
    pub force: bool,

    /// Configure the profile
    #[arg(short, long, default_value_t = false)]
    pub configure: bool,

    /// Disables the sandbox mode for the browser, linux may require this to be false
    #[arg(short, long, default_value_t = true)]
    pub sandbox: bool,

    /// Additionally returns the JSON credentials to stdout, for consumption by AWS Config [credential_process]
    #[arg(short, long, default_value_t = false)]
    pub json: bool,

    /// Enables verbose logging to the console and viewing the browser automation
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
}
