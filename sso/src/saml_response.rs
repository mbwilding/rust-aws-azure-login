use crate::helpers::base64_decode_to_string;
use anyhow::{bail, Result};
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::fmt::Display;
use std::str;

#[derive(Clone, Debug)]
pub struct Role {
    pub role_arn: String,
    pub principal_arn: String,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.role_arn, self.principal_arn)
    }
}

#[derive(Deserialize, Debug)]
struct SamlResponse {
    #[serde(rename = "Assertion")]
    assertion: Assertion,
}

#[derive(Deserialize, Debug)]
struct Assertion {
    #[serde(rename = "AttributeStatement")]
    attribute_statement: AttributeStatement,
}

#[derive(Deserialize, Debug)]
struct AttributeStatement {
    #[serde(rename = "Attribute")]
    attribute: Vec<Attribute>,
}

#[derive(Deserialize, Debug)]
struct Attribute {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "AttributeValue")]
    attribute_value: Vec<AttributeValue>,
}

#[derive(Deserialize, Debug)]
struct AttributeValue {
    #[serde(rename = "$value")]
    value: String,
}

pub fn parse_roles_from_saml_response(assertion: &str) -> Result<Vec<Role>> {
    let decoded_str = base64_decode_to_string(assertion)?;

    let saml_response: SamlResponse = from_str(&decoded_str)?;

    let mut roles = Vec::new();
    for attr in saml_response.assertion.attribute_statement.attribute {
        if attr.name == "https://aws.amazon.com/SAML/Attributes/Role" {
            for val in attr.attribute_value {
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
