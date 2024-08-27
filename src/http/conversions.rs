use crate::models::external::token::ExternalToken;
use actix_web::http::header::HeaderValue;
use anyhow::bail;

impl TryFrom<&HeaderValue> for ExternalToken {
    type Error = anyhow::Error;

    // Compiler will complain that `return Err` statements in this code are unreachable,
    // but this is not true. See test cases below.
    #[allow(unreachable_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn test_parsing_valid_token() {
        let header = HeaderValue::from_static("Bearer token");
        let token = ExternalToken::try_from(&header).unwrap();
        let string_token: String = token.into();
        assert_eq!(string_token, "token".to_string());
    }

    #[rstest]
    #[case("token")]
    #[case("My token")]
    #[case("My suer cool token")]
    #[case("")]
    #[case("Bearer")]
    fn test_parsing_invalid_token(#[case] token: &str) {
        let header = HeaderValue::from_str(token).unwrap();
        let token = ExternalToken::try_from(&header);
        assert_eq!(
            token.is_err_and(|e| e.to_string() == "Invalid token format"),
            true
        );
    }
}
