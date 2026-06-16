#[cfg(test)]
mod tests;

use actix_web::dev::ServiceRequest;
use actix_web::http::header::HeaderValue;
use anyhow::{anyhow, bail};

/// Represents an external JWT Token used to authorize the `ExternalIdentity` and issue an `InternalToken`
#[derive(Clone)]
pub struct ExternalToken {
    pub token: String,
}

/// Allows `ExternalToken` to be converted to a String
impl Into<String> for ExternalToken {
    fn into(self) -> String {
        self.token.clone()
    }
}

/// Allows a String to be converted to an `ExternalToken`
impl From<String> for ExternalToken {
    fn from(token: String) -> Self {
        ExternalToken { token }
    }
}
impl TryFrom<&HeaderValue> for ExternalToken {
    type Error = anyhow::Error;

    #[allow(unreachable_code)] // False detect
    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        match value.to_str() {
            Ok(string_value) => {
                let tokens = string_value.split(" ").collect::<Vec<&str>>();

                if tokens.len() != 2 {
                    return Err(bail!("Invalid token format"));
                }

                if tokens[0] != "Bearer" {
                    return Err(bail!("Invalid token format"));
                }

                Ok(ExternalToken::from(tokens[1].to_owned()))
            }
            Err(_) => bail!("Invalid token format"),
        }
    }
}

impl TryFrom<&ServiceRequest> for ExternalToken {
    type Error = anyhow::Error;

    fn try_from(request: &ServiceRequest) -> Result<Self, Self::Error> {
        let header = request
            .headers()
            .get("Authorization")
            .ok_or(anyhow!("Missing Authorization header"))?;
        ExternalToken::try_from(header)
    }
}
