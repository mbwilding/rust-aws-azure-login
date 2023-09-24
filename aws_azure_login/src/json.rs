use file_manager::aws_credential::AwsCredential;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JsonCredential {
    pub version: u8,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub expiration: String,
}

impl JsonCredential {
    pub fn convert(credential: AwsCredential) -> Self {
        Self {
            version: 1,
            access_key_id: credential.aws_access_key_id.unwrap(),
            secret_access_key: credential.aws_secret_access_key.unwrap(),
            session_token: credential.aws_session_token.unwrap(),
            expiration: credential
                .aws_expiration
                .unwrap()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        }
    }
}
