mod util;

use crate::util::identity_repository::IdentityRepositoryExt;
use crate::util::schema_repository::SchemaRepositoryExt;
use crate::util::test_data::{
    external_identity, external_identity_provider, external_identity_raw, external_token, principal_type, user_name,
};
use crate::util::validators::AlwaysValid;
use boxer_issuer::services::base::upsert_repository::PrincipalIdentity;
use boxer_issuer::services::principal_service::{IdentityAssociationRequest, PrincipalService};
use boxer_issuer::services::token_service::TokenProvider;
use boxer_issuer::services::token_service::TokenService;
use std::sync::Arc;
use util::principal_repository::PrincipalRepositoryExt;
use util::*;

#[tokio::test]
async fn it_can_issue_token() {
    let p_rep = principal_repository::new().with_default_data().await;
    let s_rep = schema_repository::new().with_default_data().await;
    let i_rep = identity_repository::new().with_default_data().await;
    let a_rep = principal_association_repository::new();

    let validator = Arc::new(AlwaysValid {
        identity: external_identity(),
    });

    let principal_service = Arc::new(PrincipalService::new(
        i_rep.clone(),
        p_rep.clone(),
        a_rep.clone(),
        s_rep.clone(),
    ));

    let request = IdentityAssociationRequest {
        external_identity_info: external_identity_raw(),
        principal_id: PrincipalIdentity::from((principal_type(), user_name())),
    };

    principal_service.associate(request).await.unwrap();
    let token_service = TokenService::new(
        validator,
        principal_service.clone(),
        Arc::new(vec!["dummy-secret".as_bytes()].concat()),
    );

    let token = token_service
        .issue_token(external_identity_provider(), external_token())
        .await
        .unwrap();
    assert_ne!(token, "");
}
