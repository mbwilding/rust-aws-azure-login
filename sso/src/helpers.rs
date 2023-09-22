use anyhow::Result;
use base64::engine::general_purpose;
use base64::Engine;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::Write;
use url::form_urlencoded;

pub fn base64_decode_to_string(string: &str) -> Result<String> {
    let output_vec = general_purpose::STANDARD.decode(string)?;
    Ok(String::from_utf8(output_vec)?)
}

pub fn compress_and_encode(string: &str) -> Result<Vec<u8>> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(string.as_bytes())?;
    Ok(encoder.finish()?)
}

pub fn base64_url_encode(bytes: &[u8]) -> String {
    let mut output = String::new();
    general_purpose::STANDARD.encode_string(bytes, &mut output);
    form_urlencoded::byte_serialize(output.as_bytes()).collect()
}
