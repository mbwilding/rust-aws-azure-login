use anyhow::anyhow;
use anyhow::Result;
use headless_chrome::protocol::cdp::Target::CreateTarget;
use headless_chrome::{Browser, LaunchOptions};
use maplit::hashmap;
use tracing::debug;

pub mod helpers;
mod login;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        //.json()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_line_number(true)
        .init();

    let config = aws::aws_config::AwsConfig::profile_default()?;

    let app_id_uri = match config.azure_app_id_uri {
        Some(x) => x,
        None => {
            return Err(anyhow!("azure_app_id_uri not set"));
        }
    };

    let azure_tenant_id = match config.azure_tenant_id {
        Some(x) => x,
        None => {
            return Err(anyhow!("azure_tenant_id not set"));
        }
    };

    let azure_default_username = match config.azure_default_username {
        Some(x) => x,
        None => {
            return Err(anyhow!("azure_default_username not set"));
        }
    };

    let url = login::create_login_url(&app_id_uri, &azure_tenant_id)?;
    debug!("Opening: {}", url);

    let width = 425;
    let height = 550;

    let launch_options = LaunchOptions::default_builder()
        .headless(false)
        .window_size(Some((width, height)))
        .build()
        .unwrap();

    let browser = Browser::new(launch_options)?;
    debug!("Opening new tab");

    let tab = browser.new_tab_with_options(CreateTarget {
        url: url.clone(),
        width: Some(width - 15),
        height: Some(height - 35),
        browser_context_id: None,
        enable_begin_frame_control: None,
        new_window: None,
        background: None,
    })?;

    let _ = tab.set_extra_http_headers(hashmap! {
        "Accept-Language" => "en"
    });

    tab.set_default_timeout(std::time::Duration::from_secs(10));

    // tab.enable_request_interception(|transport, session_id| hmm)?;
    // register_response_handling ???

    debug!("Waiting for sign in page to load");
    tab.wait_until_navigated()?;
    // Username
    debug!("Finding username field");
    let field = tab.wait_for_element("input#i0116")?;
    debug!("Clicking username field");
    field.click()?;
    debug!("Entering username");
    tab.send_character(&azure_default_username)?;
    debug!("Finding next button");
    let button = tab.wait_for_element("input#idSIButton9")?;
    debug!("Clicking next button");
    button.click()?;

    debug!("Waiting for password page to load");
    tab.wait_until_navigated()?;
    // Password
    debug!("Finding password field");
    let field = tab.wait_for_element("input#i0118")?;
    debug!("Clicking password field");
    field.click()?;
    debug!("Entering password");
    tab.send_character("REDACTED")?;
    debug!("Finding next button");
    let button = tab.wait_for_element("input#idSIButton9")?;
    debug!("Clicking next button");
    button.click()?;

    debug!("Finished");

    return Ok(());
}
