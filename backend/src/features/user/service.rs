use openidconnect::TokenResponse as _;

pub struct CodeUrlResult {
    pub url: String,
    pub csrf_state: String,
    pub nonce: String,
}

pub struct ExchangeResult {
    pub iss: String,
    pub sub: String,
}

#[async_trait::async_trait]
pub trait OIDCClient: Send + Sync {
    async fn code_url(&self) -> Result<CodeUrlResult, anyhow::Error>;
    async fn exchange_and_verify(
        &self,
        code: String,
        nonce: String,
    ) -> Result<ExchangeResult, anyhow::Error>;
}

type GoogleOIDCClient = openidconnect::Client<
    openidconnect::EmptyAdditionalClaims,
    openidconnect::core::CoreAuthDisplay,
    openidconnect::core::CoreGenderClaim,
    openidconnect::core::CoreJweContentEncryptionAlgorithm,
    openidconnect::core::CoreJsonWebKey,
    openidconnect::core::CoreAuthPrompt,
    openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>,
    openidconnect::StandardTokenResponse<
        openidconnect::IdTokenFields<
            openidconnect::EmptyAdditionalClaims,
            openidconnect::EmptyExtraTokenFields,
            openidconnect::core::CoreGenderClaim,
            openidconnect::core::CoreJweContentEncryptionAlgorithm,
            openidconnect::core::CoreJwsSigningAlgorithm,
        >,
        openidconnect::core::CoreTokenType,
    >,
    openidconnect::StandardTokenIntrospectionResponse<
        openidconnect::EmptyExtraTokenFields,
        openidconnect::core::CoreTokenType,
    >,
    openidconnect::core::CoreRevocableToken,
    openidconnect::StandardErrorResponse<openidconnect::RevocationErrorResponseType>,
    openidconnect::EndpointSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointNotSet,
    openidconnect::EndpointSet,
    openidconnect::EndpointMaybeSet,
    openidconnect::EndpointMaybeSet,
>;

pub struct GoogleOIDCClientImpl {
    http_client: reqwest::Client,
    client: GoogleOIDCClient,
}

impl GoogleOIDCClientImpl {
    pub async fn new(
        id: String,
        secret: String,
        issuer_url: String,
        redirect_url: String,
    ) -> Result<Self, anyhow::Error> {
        use openidconnect::core::*;
        use openidconnect::*;

        #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
        struct RevocationEndpointProviderMetadata {
            revocation_endpoint: String,
        }

        impl openidconnect::AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}

        type GoogleProviderMetadata = ProviderMetadata<
            RevocationEndpointProviderMetadata,
            CoreAuthDisplay,
            CoreClientAuthMethod,
            CoreClaimName,
            CoreClaimType,
            CoreGrantType,
            CoreJweContentEncryptionAlgorithm,
            CoreJweKeyManagementAlgorithm,
            CoreJsonWebKey,
            CoreResponseMode,
            CoreResponseType,
            CoreSubjectIdentifierType,
        >;

        let id = ClientId::new(id);
        let secret = ClientSecret::new(secret);
        let issuer_url = IssuerUrl::new(issuer_url)?;

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        let provider_metadata =
            GoogleProviderMetadata::discover_async(issuer_url, &http_client).await?;

        let revocation_endpoint = provider_metadata
            .additional_metadata()
            .revocation_endpoint
            .clone();

        let client = CoreClient::from_provider_metadata(provider_metadata, id, Some(secret))
            .set_redirect_uri(RedirectUrl::new(redirect_url)?)
            .set_revocation_url(RevocationUrl::new(revocation_endpoint)?);

        Ok(Self {
            http_client,
            client,
        })
    }
}

#[async_trait::async_trait]
impl OIDCClient for GoogleOIDCClientImpl {
    async fn code_url(&self) -> Result<CodeUrlResult, anyhow::Error> {
        let (authorize_url, csrf_state, nonce) = self.client
            .authorize_url(
                openidconnect::AuthenticationFlow::<openidconnect::core::CoreResponseType>::AuthorizationCode,
                openidconnect::CsrfToken::new_random,
                openidconnect::Nonce::new_random,
            )
            .url();

        Ok(CodeUrlResult {
            url: authorize_url.to_string(),
            csrf_state: csrf_state.secret().clone(),
            nonce: nonce.secret().clone(),
        })
    }

    async fn exchange_and_verify(
        &self,
        code: String,
        nonce: String,
    ) -> Result<ExchangeResult, anyhow::Error> {
        let token_response = self
            .client
            .exchange_code(openidconnect::AuthorizationCode::new(code))?
            .request_async(&self.http_client)
            .await?;

        let id_token = token_response
            .id_token()
            .ok_or_else(|| anyhow::anyhow!("ID token not found"))?;

        let nonce = openidconnect::Nonce::new(nonce);
        let claims = id_token.claims(&self.client.id_token_verifier(), &nonce)?;

        Ok(ExchangeResult {
            iss: claims.issuer().to_string(),
            sub: claims.subject().to_string(),
        })
    }
}
