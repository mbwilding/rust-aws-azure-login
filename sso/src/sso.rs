use crate::filter::{handler, Filters};
use crate::saml_request::create_login_url;
use crate::saml_response::{parse_roles_from_saml_response, Role};
use anyhow::{anyhow, bail, Result};
use aws_sdk_sts::config::Region;
use aws_smithy_types::date_time::Format;
use chrono::Utc;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use directories::UserDirs;
use file_manager::aws_config::AwsConfig;
use file_manager::aws_credentials::AwsCredentials;
use headless_chrome::protocol::cdp::Target::CreateTarget;
use headless_chrome::{Browser, LaunchOptions};
use maplit::hashmap;
use shared::args::Args;
use std::collections::HashMap;
use tracing::debug;

pub async fn login(
    configs: &HashMap<String, AwsConfig>,
    credentials: &mut HashMap<String, AwsCredentials>,
    profile_name: &str,
    force_refresh: bool,
    no_prompt: bool,
    args: &Args,
) -> Result<AwsCredentials> {
    if !force_refresh {
        let credential = AwsCredentials::get(profile_name, credentials);
        if credential.is_ok() {
            let credential = credential.unwrap();
            if !credential.is_profile_about_to_expire() {
                return Ok(credential);
            }
        }
    }

    let profile = AwsConfig::get(profile_name, configs)?;

    println!("Logging into profile: {}", profile_name);

    let saml = perform_login(&profile, args)?;

    let roles = parse_roles_from_saml_response(&saml)?;

    let (role, duration_hours) = role_and_duration(
        roles,
        no_prompt,
        profile.azure_default_role_arn.clone(),
        profile.azure_default_duration_hours,
    )?;

    let credentials = assume_role(
        profile_name,
        &saml,
        &role,
        duration_hours,
        profile.region.to_owned(),
    )
    .await?;

    Ok(credentials)
}

fn perform_login(profile: &AwsConfig, args: &Args) -> Result<String> {
    let width = 425;
    let height = 550;

    let mut launch_options = LaunchOptions::default_builder();

    launch_options
        .headless(!args.debug)
        .sandbox(args.sandbox)
        .window_size(Some((width, height)));

    if profile.azure_default_remember_me == Some(true) {
        let user_data_path = match UserDirs::new() {
            Some(user_dirs) => user_dirs.home_dir().join(".aws/chromium"),
            None => Err(anyhow!("Unable to get user directories"))?,
        };
        launch_options.user_data_dir(Some(user_data_path));
    }

    let launch_options_built = launch_options.build()?;
    let browser = Browser::new(launch_options_built)?;

    let azure_url = create_login_url(profile)?;

    let tab = browser.new_tab_with_options(CreateTarget {
        url: azure_url.clone(),
        width: Some(width - 15),
        height: Some(height - 35),
        browser_context_id: None,
        enable_begin_frame_control: None,
        new_window: Some(false),
        background: None,
    })?;

    tab.set_extra_http_headers(hashmap! {
        "Accept-Language" => "en"
    })?;

    tab.set_default_timeout(std::time::Duration::from_secs(100));

    // TODO: Cover all login cases and fix response handling
    if false {
        let filters = Filters {
            urls: vec![azure_url, "amazon".to_string()],
        };

        tab.register_response_handling(
            "saml",
            Box::new(move |params, get_response_body| {
                handler(params, get_response_body, &filters);
            }),
        )?;

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

fn role_and_duration(
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
            .with_prompt("Role")
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
    role: &Role,
    duration_hours: u8,
    region: Option<String>,
) -> Result<AwsCredentials> {
    let config = if let Some(region_str) = region {
        aws_config::from_env()
            .region(Region::new(region_str))
            .no_credentials()
            .load()
            .await
    } else {
        aws_config::from_env().no_credentials().load().await
    };

    let sts_client = aws_sdk_sts::Client::new(&config);

    let duration_seconds = (duration_hours as i32) * 60 * 60;

    let assume_role_request = sts_client
        .assume_role_with_saml()
        .role_arn(&role.role_arn)
        .principal_arn(&role.principal_arn)
        .saml_assertion(assertion)
        .duration_seconds(duration_seconds);

    let assume_role_response = assume_role_request.send().await?;

    let credentials = assume_role_response
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
