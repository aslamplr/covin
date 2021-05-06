use biscuit::{jwa::SignatureAlgorithm, jwk::JWKSet, Empty, JWT};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use thiserror::Error;

static JWK_SET: OnceCell<JWKSet<Empty>> = OnceCell::new();

async fn get_jwk_set<'t>() -> Result<&'t OnceCell<JWKSet<Empty>>, VerifierError> {
    if JWK_SET.get().is_none() {
        let region_str = std::env::var("AWS_COGNITO_REGION").unwrap();
        let pool_id_str = std::env::var("AWS_COGNITO_POOL_ID").unwrap();
        let jwks_url = format!(
            "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
            region_str, pool_id_str
        );
        let _iss = format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            region_str, pool_id_str
        );

        let jwk_set = reqwest::get(jwks_url)
            .await?
            .json::<JWKSet<Empty>>()
            .await?;

        #[allow(unused_must_use)]
        {
            JWK_SET.set(jwk_set);
        }
    }

    Ok(&JWK_SET)
}

async fn validate_decode_jwt(jwt: &str) -> Result<PrivateClaims, VerifierError> {
    let jwks = get_jwk_set().await?.get().unwrap();
    let encoded_token = JWT::<PrivateClaims, Empty>::new_encoded(&jwt);

    let decoded_token = encoded_token
        .decode_with_jwks(&jwks, Some(SignatureAlgorithm::RS256))
        .map_err(VerifierError::from)?;
    decoded_token.validate(biscuit::ValidationOptions::default())?;
    Ok(decoded_token.payload()?.private.to_owned())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PrivateClaims {
    event_id: String,
    scope: String,
    client_id: String,
    username: String,
}

#[derive(Debug, Error)]
pub enum VerifierError {
    #[error("unable to get jwks")]
    JWKSGet(#[from] reqwest::Error),
    #[error("unable to decode jwt")]
    BiscuitError(#[from] biscuit::errors::Error),
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("Unable to verify")]
    VerifierError(#[from] VerifierError),
}

#[derive(Debug, Default)]
pub struct AuthClaims {
    pub user_id: String,
    pub client_id: Option<String>,
    pub event_id: Option<String>,
    pub scope: Option<String>,
}

impl From<PrivateClaims> for AuthClaims {
    fn from(
        PrivateClaims {
            username,
            client_id,
            event_id,
            scope,
        }: PrivateClaims,
    ) -> Self {
        AuthClaims {
            user_id: username,
            client_id: Some(client_id),
            event_id: Some(event_id),
            scope: Some(scope),
        }
    }
}

#[derive(Debug)]
pub struct AuthToken<'tkn> {
    token: &'tkn str,
}

impl AuthToken<'_> {
    async fn decode(&self) -> Result<AuthClaims, AuthError> {
        let tokens = self.token.split(' ').collect::<Vec<&str>>();
        if tokens.len().ne(&2) {
            return Err(AuthError::InvalidCredentials);
        }
        match (tokens[0], tokens[1]) {
            ("Bearer", token) => {
                let claims = validate_decode_jwt(token).await?;
                Ok(claims.into())
            }
            _ => Err(AuthError::InvalidCredentials),
        }
    }
}

impl<'tkn, T> From<&'tkn T> for AuthToken<'tkn>
where
    T: AsRef<str>,
{
    fn from(t: &'tkn T) -> Self {
        Self { token: t.as_ref() }
    }
}

pub async fn decode_token(token: &str) -> Result<AuthClaims, AuthError> {
    let token = AuthToken::from(&token);
    tracing::debug!(message = "token", token = ?token);
    match token.decode().await {
        Ok(claims) => Ok(claims),
        Err(err) => {
            tracing::error!(error = ?err, message = "Authentication Error");
            Err(err)
        }
    }
}
