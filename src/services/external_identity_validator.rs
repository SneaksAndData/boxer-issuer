use async_trait::async_trait;
use std::error::Error;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct ExternalIdentity {
    pub user_id: String,
    pub identity_provider: String,
}

#[async_trait]
pub trait ExternalIdentityValidator {
    async fn validate(&self, token: &str) -> Result<ExternalIdentity, Box<dyn Error>>;
}

pub struct ExternalIdentityValidatorImpl;

#[async_trait]
impl ExternalIdentityValidator for ExternalIdentityValidatorImpl {
    async fn validate(&self, token: &str) -> Result<ExternalIdentity, Box<dyn Error>> {
        Ok(ExternalIdentity {
            user_id: token.to_string(),
            identity_provider: "google".to_string(),
        })
    }
}

// The convention is to create a module named tests in each file to contain the
// test functions and to annotate the module with cfg(test).
// see: https://doc.rust-lang.org/book/ch11-03-test-organization.html#unit-tests
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validator_validates() {
        let validator = ExternalIdentityValidatorImpl;

        // It's a unit test, so it's okay to use unwrap here.
        let result = validator.validate("123").await.unwrap();
        assert_eq!(
            result,
            ExternalIdentity {
                user_id: "123".to_string(),
                identity_provider: "google".to_string(),
            }
        );
    }
}
