use crate::helpers::{base64_url_encode, compress_and_encode};
use anyhow::{anyhow, Result};
use aws::aws_config::AwsConfig;
use aws::aws_credentials::AwsCredentials;
use chrono::Utc;
use headless_chrome::protocol::cdp::Target::CreateTarget;
use headless_chrome::{Browser, LaunchOptions};
use maplit::hashmap;
use tracing::debug;
use uuid::Uuid;

pub fn create_login_url(config: &AwsConfig) -> Result<String> {
    let assertion_consumer_service_url = match &config.region {
        Some(r) if r.starts_with("us-gov") => {
            "https://signin.amazonaws-us-gov.com/saml".to_string()
        }
        Some(r) if r.starts_with("cn-") => "https://signin.amazonaws.cn/saml".to_string(),
        _ => "https://signin.aws.amazon.com/saml".to_string(),
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

pub fn login(profile_name: &str, _aws_no_verify_ssl: bool, _no_prompt: bool) -> Result<()> {
    let profile = AwsConfig::profile(profile_name)?;

    let _saml = perform_login(profile)?;

    // let roles = parse_roles_from_saml_response(&saml);

    // let (rl, duration_hours) = ask_user_for_role_and_duration(
    //     &roles,
    //     no_prompt,
    //     &profile.azure_default_role_arn,
    //     &profile.azure_default_duration_hours,
    // );

    // assume_role(
    //     profile_name,
    //     &saml,
    //     &rl,
    //     duration_hours,
    //     aws_no_verify_ssl,
    //     &profile.region,
    // );

    Ok(())
}

pub fn login_all(force_refresh: bool, aws_no_verify_ssl: bool, no_prompt: bool) -> Result<()> {
    let all_profiles = AwsConfig::profiles()?;

    for profile in all_profiles.iter() {
        let profile_name = profile.0.as_str();
        let credentials = AwsCredentials::profile(profile_name).unwrap();

        if force_refresh && credentials.is_profile_about_to_expire() {
            let _ = login(profile_name, aws_no_verify_ssl, no_prompt);
        }
    }

    Ok(())
}

pub fn perform_login(profile: AwsConfig) -> Result<String> {
    let width = 425;
    let height = 550;

    let launch_options = LaunchOptions::default_builder()
        .headless(false) // TODO: true in production
        .window_size(Some((width, height)))
        .build()?;

    let browser = Browser::new(launch_options)?;

    let tab = browser.new_tab_with_options(CreateTarget {
        url: create_login_url(&profile)?,
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
    tab.send_character(
        &profile
            .azure_default_username
            .ok_or(anyhow!("azure_default_username not set"))?,
    )?;
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
    tab.send_character("TODO: Securely saved password")?;
    debug!("Finding next button");
    let button = tab.wait_for_element("input#idSIButton9")?;
    debug!("Clicking next button");
    button.click()?;

    debug!("Finished");

    Ok("TODO: SAML Response".to_string())
}
