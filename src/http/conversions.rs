use actix_web::http::header::HeaderValue;
use anyhow::bail;
use crate::models::external::token::ExternalToken;

impl TryFrom<&HeaderValue> for ExternalToken {
    type Error = anyhow::Error;

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
            Err(_) => Err(bail!("Invalid token format")),
        }
    }
}
