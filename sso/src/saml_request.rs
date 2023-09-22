use anyhow::anyhow;
use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use chrono::Utc;
use file_manager::aws_config::AwsConfig;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::Write;
use url::form_urlencoded;
use uuid::Uuid;

pub fn create_login_url(config: &AwsConfig) -> Result<String> {
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

fn compress_and_encode(string: &str) -> Result<Vec<u8>> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(string.as_bytes())?;
    Ok(encoder.finish()?)
}

pub fn base64_url_encode(bytes: &[u8]) -> String {
    let mut output = String::new();
    general_purpose::STANDARD.encode_string(bytes, &mut output);
    form_urlencoded::byte_serialize(output.as_bytes()).collect()
}
