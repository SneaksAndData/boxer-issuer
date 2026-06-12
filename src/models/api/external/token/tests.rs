use crate::models::api::external::token::ExternalToken;
use actix_web::http::header::HeaderValue;
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
    assert_eq!(token.is_err_and(|e| e.to_string() == "Invalid token format"), true);
}
