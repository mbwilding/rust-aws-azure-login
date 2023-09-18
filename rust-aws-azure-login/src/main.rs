use chromiumoxide::{Browser, BrowserConfig};
use maplit::hashmap;
use tracing::debug;

pub mod helpers;
mod login;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        //.json()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_line_number(true)
        .init();

    let config = aws::aws_config::AwsConfig::profile_default();

    let url = login::create_login_url(
        "https://signin.aws.amazon.com/saml",
        "308d2158-f6df-4747-8791-e970657274d5",
    )?;
    debug!("Opening: {}", url);

    let width = 425;
    let height = 550;

    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head()
            .window_size(width, height)
            .build()?,
    )
    .await?;

    debug!("Opening new page");
    let page = browser.new_page(&url).await?;

    // Username
    debug!("Waiting for sign in page to load");
    page.wait_for_navigation().await?;
    debug!("Finding username field");
    let field = page.find_element("input#i0116").await?;
    debug!("Clicking username field");
    field.click().await?;
    debug!("Entering username");
    field
        .type_str(&config?.azure_default_username.unwrap())
        .await?;
    debug!("Finding next button");
    let button = page.find_element("input#idSIButton9").await?;
    debug!("Clicking next button");
    button.click().await?;

    // Password
    debug!("Waiting for password page to load");
    page.wait_for_navigation().await?;
    debug!("Finding password field");
    let field = page.find_element("input#i0118").await?;
    debug!("Clicking password field");
    field.click().await?;
    debug!("Entering password");
    field.type_str("REDACTED").await?;
    debug!("Finding next button");
    let button = page.find_element("input#idSIButton9").await?;
    debug!("Clicking next button");
    button.click().await?;

    debug!("Finished");

    return Ok(());
}
