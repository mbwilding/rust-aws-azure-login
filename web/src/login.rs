use crate::helpers::{base64_url_encode, compress_and_encode};
use crate::saml_response::{parse_roles_from_saml_response, Role};
use anyhow::{anyhow, bail, Result};
use aws::aws_config::AwsConfig;
use aws::aws_credentials::AwsCredentials;
use aws_sdk_sts::config::Region;
use aws_smithy_types::date_time::Format;
use chrono::Utc;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use headless_chrome::browser::tab::{RequestInterceptor, RequestPausedDecision};
use headless_chrome::browser::transport::{SessionId, Transport};
use headless_chrome::protocol::cdp::Fetch::events::RequestPausedEvent;
use headless_chrome::protocol::cdp::Network::GetResponseBody;
use headless_chrome::protocol::cdp::Target::CreateTarget;
use headless_chrome::{Browser, LaunchOptions};
use maplit::hashmap;
use scraper::{Html, Selector};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::debug;
use uuid::Uuid;

fn create_login_url(config: &AwsConfig) -> Result<String> {
    let assertion_consumer_service_url = match &config.region {
        Some(r) if r.starts_with("us-gov") => "https://signin.amazonaws-us-gov.com/saml",
        Some(r) if r.starts_with("cn-") => "https://signin.amazonaws.cn/saml",
        _ => "https://signin.aws.amazon.com/saml",
    };

    let saml_request = format!(
        r#"
        <samlp:AuthnRequest xmlns="urn:oasis:names:tc:SAML:2.0:metadata" ID="id{}" Version="2.0" IssueInstant="{}" IsPassive="false" AssertionConsumerServiceURL="{}" xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol">
            <Issuer xmlns="urn:oasis:names:tc:SAML:2.0:assertion">{}</Issuer>
            <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"></samlp:NameIDPolicy>
        </samlp:AuthnRequest>
        "#,
        Uuid::new_v4(),
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        assertion_consumer_service_url,
        config
            .azure_app_id_uri
            .as_ref()
            .ok_or(anyhow!("azure_app_id_uri not set"))?,
    );

    let compressed_bytes = compress_and_encode(&saml_request)?;
    let saml_base64_encoded = base64_url_encode(&compressed_bytes);

    let url = format!(
        "https://login.microsoftonline.com/{}/saml2?SAMLRequest={}",
        config
            .azure_tenant_id
            .as_ref()
            .ok_or(anyhow!("azure_tenant_id not set"))?,
        saml_base64_encoded
    );

    Ok(url)
}

pub async fn login(
    profile_name: &str,
    force_refresh: bool,
    no_prompt: bool,
) -> Result<AwsCredentials> {
    let profile = &AwsConfig::profile(profile_name)?;

    // TODO: This will cause an IO write to the creds, even if no change
    if !force_refresh {
        let credentials = AwsCredentials::profile(profile_name)?;
        if !credentials.is_profile_about_to_expire() {
            return Ok(credentials);
        }
    }

    let saml = perform_login(profile)?;

    let roles = parse_roles_from_saml_response(&saml)?;

    let (role, duration_hours) = ask_user_for_role_and_duration(
        roles,
        no_prompt,
        profile.azure_default_role_arn.clone(),
        profile.azure_default_duration_hours,
    )?;

    let credentials = assume_role(
        profile_name,
        &saml,
        role,
        duration_hours,
        profile.region.to_owned(),
    )
    .await?;

    Ok(credentials)
}

pub async fn login_all(force_refresh: bool, no_prompt: bool) -> Result<Vec<AwsCredentials>> {
    let all_profiles = AwsConfig::profiles()?;

    let mut profiles_to_refresh = Vec::new();

    for profile in all_profiles.iter() {
        let profile_name = profile.0.as_str();
        let credentials = AwsCredentials::profile(profile_name)?;

        if force_refresh || credentials.is_profile_about_to_expire() {
            let credentials = login(profile_name, force_refresh, no_prompt).await?;
            profiles_to_refresh.push(credentials);
        }
    }

    Ok(profiles_to_refresh)
}

fn perform_login(profile: &AwsConfig) -> Result<String> {
    let width = 425;
    let height = 550;

    let path = PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe");

    let launch_options = LaunchOptions::default_builder()
        .path(Some(path))
        .headless(false) // TODO: true in production
        .window_size(Some((width, height)))
        .user_data_dir(Some(PathBuf::from(
            r"C:\Users\mbwil\AppData\Local\Google\Chrome\User Data",
        )))
        .build()?;

    let browser = Browser::new(launch_options)?;

    let tab = browser.new_tab_with_options(CreateTarget {
        url: create_login_url(profile)?,
        width: Some(width - 15),
        height: Some(height - 35),
        browser_context_id: None,
        enable_begin_frame_control: None,
        new_window: None,
        background: None,
    })?;

    tab.set_extra_http_headers(hashmap! {
        "Accept-Language" => "en"
    })?;

    tab.set_default_timeout(std::time::Duration::from_secs(10));

    if false {
        // Username
        debug!("Waiting for sign in page to load");
        tab.wait_until_navigated()?;
        debug!("Finding username field");
        let field = tab.wait_for_element("input#i0116")?;
        debug!("Clicking username field");
        field.click()?;
        debug!("Entering username");
        tab.send_character(
            profile
                .azure_default_username
                .as_ref()
                .ok_or(anyhow!("azure_default_username not set"))?,
        )?;
        debug!("Finding next button");
        let button = tab.wait_for_element("input#idSIButton9")?;
        debug!("Clicking next button");
        button.click()?;

        // Password
        debug!("Waiting for password page to load");
        tab.wait_until_navigated()?;
        debug!("Finding password field");
        let field = tab.wait_for_element("input#i0118")?;
        debug!("Clicking password field");
        field.click()?;
        debug!("Entering password");
        tab.send_character("TODO: Securely saved password")?;
        debug!("Finding next button");
        let button = tab.wait_for_element("input#idSIButton9")?;
        debug!("Clicking next button");
        button.click()?;

        debug!("Finished");
    }

    let saml_response = tab.wait_for_element("form#saml_form > input[name=SAMLResponse]")?;
    let saml = saml_response.get_attribute_value("value")?.unwrap();

    Ok(saml)
}

fn ask_user_for_role_and_duration(
    roles: Vec<Role>,
    no_prompt: bool,
    default_role_arn: Option<String>,
    default_duration_hours: Option<u8>,
) -> Result<(Role, u8)> {
    let mut duration_hours: u8 = default_duration_hours.unwrap_or_default();

    let selected_role = if roles.is_empty() {
        bail!("No roles found in SAML response.");
    } else if roles.len() == 1 {
        roles.first().unwrap().to_owned()
    } else if no_prompt {
        if let Some(ref arn) = default_role_arn {
            if let Some(role) = roles.iter().find(|r| &r.role_arn == arn) {
                role.to_owned()
            } else {
                bail!("No role matching the default role ARN found in the SAML response.");
            }
        } else {
            bail!("No default role ARN provided and multiple roles found in SAML response.");
        }
    } else {
        let selection = Select::new()
            .with_prompt("Role:")
            .default(0)
            .items(
                &roles
                    .iter()
                    .map(|r| r.role_arn.as_str())
                    .collect::<Vec<_>>(),
            )
            .interact()
            .unwrap();

        roles[selection].to_owned()
    };

    if !no_prompt || default_duration_hours.is_none() {
        duration_hours = loop {
            let input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Default Session Duration Hours (up to 12)")
                .default(default_duration_hours.map_or(String::new(), |x| x.to_string()))
                .allow_empty(true)
                .interact_text()
                .unwrap();

            if let Ok(value) = input.parse::<u8>() {
                if value > 0 && value <= 12 {
                    break value;
                }
            }
        };
    }

    Ok((selected_role, duration_hours))
}

async fn assume_role(
    profile_name: &str,
    assertion: &str,
    role: Role,
    duration_hours: u8,
    region: Option<String>,
) -> Result<AwsCredentials> {
    let config = if let Some(region_str) = region {
        aws_config::from_env()
            .region(Region::new(region_str))
            .load()
            .await
    } else {
        aws_config::from_env().load().await
    };

    let sts_client = aws_sdk_sts::Client::new(&config);

    let duration_seconds = (duration_hours as i32) * 60 * 60;

    let assume_role_request = sts_client
        .assume_role_with_saml()
        .role_arn(role.role_arn)
        .principal_arn(role.principal_arn)
        .saml_assertion(assertion)
        .duration_seconds(duration_seconds);

    let assume_role_output = assume_role_request.send().await?;

    let credentials = assume_role_output
        .credentials
        .ok_or(anyhow!("No credentials found in assume role response"))?;

    let access_key_id = credentials
        .access_key_id
        .ok_or(anyhow!("No access key ID found in assume role response"))?;

    let secret_access_key = credentials.secret_access_key.ok_or(anyhow!(
        "No secret access key found in assume role response"
    ))?;

    let session_token = credentials
        .session_token
        .ok_or(anyhow!("No session token found in assume role response"))?;

    let expiration = credentials
        .expiration
        .ok_or(anyhow!("No expiration found in assume role response"))?
        .fmt(Format::DateTime)?;

    let expiration = chrono::DateTime::parse_from_rfc3339(&expiration)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| anyhow!("Failed to parse datetime: {:?}", e))?;

    Ok(AwsCredentials {
        profile_name: Some(profile_name.to_owned()),
        aws_access_key_id: Some(access_key_id),
        aws_secret_access_key: Some(secret_access_key),
        aws_session_token: Some(session_token),
        aws_expiration: Some(expiration),
    })
}
