use crate::helpers::{base64_url_encode, compress_and_encode};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

pub fn create_login_url(app_id_uri: &str, tenant_id: &str) -> Result<String> {
    let saml_request = format!(
        r#"
        <samlp:AuthnRequest xmlns="urn:oasis:names:tc:SAML:2.0:metadata" ID="id{}" Version="2.0" IssueInstant="{}" IsPassive="false" AssertionConsumerServiceURL="{}" xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol">
            <Issuer xmlns="urn:oasis:names:tc:SAML:2.0:assertion">{}</Issuer>
            <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"></samlp:NameIDPolicy>
        </samlp:AuthnRequest>
        "#,
        Uuid::new_v4().to_string(),
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        app_id_uri,
        app_id_uri
    );

    let compressed_bytes = compress_and_encode(&saml_request)?;
    let saml_base64_encoded = base64_url_encode(&compressed_bytes);

    let url = format!(
        "https://login.microsoftonline.com/{}/saml2?SAMLRequest={}",
        tenant_id, saml_base64_encoded
    );

    Ok(url)
}
