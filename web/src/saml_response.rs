extern crate base64;
extern crate serde;
extern crate serde_xml_rs;

use crate::helpers::base64_decode_to_string;
use anyhow::{bail, Result};
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::str;

#[derive(Deserialize, Clone)]
pub struct Role {
    pub(crate) role_arn: String,
    pub(crate) principal_arn: String,
}

#[derive(Deserialize)]
struct AttributeValue {
    value: String,
}

#[derive(Deserialize)]
struct Attribute {
    name: String,
    attribute_values: Vec<AttributeValue>,
}

#[derive(Deserialize)]
struct AttributeStatement {
    attributes: Vec<Attribute>,
}

#[derive(Deserialize)]
struct Assertion {
    attribute_statement: AttributeStatement,
}

#[derive(Deserialize)]
struct SamlResponse {
    assertion: Assertion,
}

pub fn parse_roles_from_saml_response(assertion: &str) -> Result<Vec<Role>> {
    let decoded_str = base64_decode_to_string(assertion)?;

    let saml_response: SamlResponse = from_str(&decoded_str)?;

    let mut roles = Vec::new();
    for attr in saml_response.assertion.attribute_statement.attributes {
        if attr.name == "https://aws.amazon.com/SAML/Attributes/Role" {
            for val in attr.attribute_values {
                let parts: Vec<&str> = val.value.split(',').collect();
                if parts.len() < 2 {
                    bail!("Malformed role data");
                }
                let role = if parts[0].contains(":role/") {
                    Role {
                        role_arn: parts[0].trim().to_string(),
                        principal_arn: parts[1].trim().to_string(),
                    }
                } else {
                    Role {
                        role_arn: parts[1].trim().to_string(),
                        principal_arn: parts[0].trim().to_string(),
                    }
                };
                roles.push(role);
            }
        }
    }

    Ok(roles)
}
