use crate::http::errors::*;
use crate::models::external::identity_provider::ExternalIdentityProvider;
use crate::models::external::token::ExternalToken;
use crate::services::token_service::{TokenProvider, TokenService};
use actix_web::web::{Data, Path};
use actix_web::{get, HttpRequest};
use std::sync::Arc;

#[utoipa::path(responses((status = OK)))]
#[get("/token/{identity_provider}")]
pub async fn token(data: Data<Arc<TokenService>>, identity_provider: Path<String>, req: HttpRequest) -> Result<String> {
    let ip = ExternalIdentityProvider::from(identity_provider.to_string());
    let maybe_header = req.headers().get("Authorization").map(|header| async move {
        let token = ExternalToken::try_from(header)?;
        data.issue_token(ip, token).await
    });

    match maybe_header {
        Some(fut) => {
            let token = fut.await?;
            Ok(token)
        }
        None => Err(Error::new("Unauthorized")),
    }
}
