use crate::saml_request::create_login_url;
use crate::saml_response::{parse_roles_from_saml_response, Role};
use anyhow::{anyhow, bail, Result};
use aws_sdk_sts::config::Region;
use aws_smithy_types::date_time::Format;
use chrono::Utc;
use crossbeam::channel;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use directories::UserDirs;
use file_manager::aws_config::AwsConfig;
use file_manager::aws_credential::AwsCredential;
use headless_chrome::browser::tab::RequestPausedDecision;
use headless_chrome::browser::transport::{SessionId, Transport};
use headless_chrome::protocol::cdp::Fetch::events::RequestPausedEvent;
use headless_chrome::protocol::cdp::Fetch::{RequestPattern, RequestStage};
use headless_chrome::protocol::cdp::Target::CreateTarget;
use headless_chrome::{Browser, LaunchOptions};
use log::info;
use maplit::hashmap;
use shared::args::Args;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use url::form_urlencoded;

pub async fn login(
    configs: &HashMap<String, AwsConfig>,
    credentials: &mut HashMap<String, AwsCredential>,
    profile_name: &str,
    args: &Args,
) -> Result<AwsCredential> {
    if !args.force {
        let credential = AwsCredential::get(profile_name, credentials);
        if credential.is_ok() {
            let credential = credential.unwrap();
            if !credential.is_profile_about_to_expire() {
                return Ok(credential);
            }
        }
    }

    let config = AwsConfig::get(profile_name, configs)?;

    info!("Logging into profile: {}", profile_name);

    let saml = perform_login(&config, args)?;
    let roles = parse_roles_from_saml_response(&saml)?;

    let (role, duration_hours) = role_and_duration(
        roles,
        config.azure_default_role_arn.clone(),
        config.azure_default_duration_hours,
    )?;

    let credential = assume_role(
        profile_name,
        &saml,
        &role,
        duration_hours,
        config.region.to_owned(),
    )
    .await?;

    AwsCredential::upsert(profile_name, &credential, credentials)?;
    AwsCredential::write(credentials)?;

    Ok(credential)
}

pub async fn login_all(
    configs: &HashMap<String, AwsConfig>,
    credentials: &mut HashMap<String, AwsCredential>,
    args: &Args,
) -> Result<()> {
    for profile_name in configs.keys() {
        login(configs, credentials, profile_name, args).await?;
    }

    Ok(())
}

fn perform_login(profile: &AwsConfig, args: &Args) -> Result<String> {
    let mut saml_response_result = saml_sso_fetch(profile, args, !args.debug);

    if saml_response_result.is_err() {
        // TODO: Make `saml_sso_fetch` return an error early if asking for details
        saml_response_result = saml_sso_fetch(profile, args, false);
    }

    saml_response_result
}

fn saml_sso_fetch(profile: &AwsConfig, args: &Args, headless: bool) -> Result<String> {
    let width = 425;
    let height = 550;

    let mut launch_options = LaunchOptions::default_builder();

    launch_options
        .headless(headless)
        .sandbox(args.sandbox)
        .window_size(Some((width, height)))
        .idle_browser_timeout(Duration::from_secs(3600)); // TODO: Revise

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

    tab.stop_loading()?; // TODO: Part 1 for interception hack, if already logged in it doesn't detect the response unless you reload the browser

    tab.set_extra_http_headers(hashmap! {
        "Accept-Language" => "en"
    })?;

    let aws_url = profile.azure_app_id_uri.clone().unwrap();
    let patterns = vec![RequestPattern {
        url_pattern: Some(aws_url.clone()),
        resource_Type: None,
        request_stage: Some(RequestStage::Response),
    }];
    tab.enable_fetch(Some(&patterns), None)?;

    let (sender, receiver) = channel::bounded(1);

    tab.enable_request_interception(Arc::new(
        move |_transport: Arc<Transport>,
              _session_id: SessionId,
              intercepted: RequestPausedEvent| {
            if intercepted.params.request.url.contains(&aws_url) {
                let response_data = intercepted.params.request.post_data.unwrap();
                sender.send(response_data).unwrap();
            }

            RequestPausedDecision::Continue(None)
        },
    ))?;

    tab.reload(false, None)?; // TODO: Part 2 for interception hack, if already logged in it doesn't detect the response unless you reload the browser

    let saml_response = receiver
        .recv()?
        .strip_prefix("SAMLResponse=")
        .unwrap()
        .to_string();

    let saml_response_decoded = form_urlencoded::parse(saml_response.as_bytes())
        .map(|(key, _)| key)
        .collect();

    tab.wait_until_navigated()?; // TODO: Allows time for the remember me response to go through and set the cookies

    Ok(saml_response_decoded)
}

fn role_and_duration(
    roles: Vec<Role>,
    default_role_arn: Option<String>,
    default_duration_hours: Option<u8>,
) -> Result<(Role, u8)> {
    let mut duration_hours: u8 = default_duration_hours.unwrap_or_default();

    let selected_role = if roles.is_empty() {
        bail!("No roles found in SAML response.");
    } else if roles.len() == 1 {
        roles.first().unwrap().to_owned()
    } else if let Some(default_role_arn) = &default_role_arn {
        get_default_role(&roles, &Some(default_role_arn.clone()))?
    } else {
        select_role_interactively(&roles)
    };

    if default_duration_hours.is_none() {
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

fn get_default_role(roles: &[Role], default_role_arn: &Option<String>) -> Result<Role> {
    match default_role_arn {
        Some(arn) => roles
            .iter()
            .find(|r| &r.role_arn == arn)
            .cloned()
            .ok_or_else(|| {
                anyhow!("No role matching the default role ARN found in the SAML response.")
            }),
        None => bail!("No default role ARN provided and multiple roles found in SAML response."),
    }
}

fn select_role_interactively(roles: &[Role]) -> Role {
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
}

async fn assume_role(
    profile_name: &str,
    assertion: &str,
    role: &Role,
    duration_hours: u8,
    region: Option<String>,
) -> Result<AwsCredential> {
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

    Ok(AwsCredential {
        profile_name: Some(profile_name.to_owned()),
        aws_access_key_id: Some(access_key_id),
        aws_secret_access_key: Some(secret_access_key),
        aws_session_token: Some(session_token),
        aws_expiration: Some(expiration),
    })
}
