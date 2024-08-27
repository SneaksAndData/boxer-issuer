pub struct OidcExternalIdentityProviderSettings {
    /// The claim that contains the user id (or name) in the external token.
    /// This is used to extract the user id from the token and issue the internal token with
    /// policy based on external identity.
    pub user_id_claim: String,

    /// The well known uri of the identity provider.
    /// This is used to get the public key to validate the token.
    pub discovery_url: String,

    /// The list of issuers that are allowed to issue tokens.
    pub issuers: Vec<String>,

    /// The list of audiences that are allowed to consume tokens.
    pub audiences: Vec<String>,
}
