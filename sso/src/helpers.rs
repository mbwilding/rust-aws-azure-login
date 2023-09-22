use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;

pub fn base64_decode_to_string(string: &str) -> Result<String> {
    let output_vec = general_purpose::STANDARD.decode(string)?;
    Ok(String::from_utf8(output_vec)?)
}
