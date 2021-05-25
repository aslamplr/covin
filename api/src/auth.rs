use biscuit::{jwa::SignatureAlgorithm, jwk::JWKSet, Empty, Validation, ValidationOptions, JWT};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use thiserror::Error;
use warp_lambda::lambda_http::request::RequestContext;

static JWKS_URL: Lazy<JwksIssUrls> = Lazy::new(JwksIssUrls::init);
static JWK_SET: OnceCell<JWKSet<Empty>> = OnceCell::new();

struct JwksIssUrls {
    jwks_url: String,
    iss: String,
}

impl JwksIssUrls {
    fn init() -> Self {
        let region_str = std::env::var("AWS_COGNITO_REGION").unwrap();
        let pool_id_str = std::env::var("AWS_COGNITO_POOL_ID").unwrap();
        let jwks_url = format!(
            "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
            region_str, pool_id_str
        );
        let iss = format!(
            "https://cognito-idp.{}.amazonaws.com/{}",
            region_str, pool_id_str
        );
        Self { jwks_url, iss }
    }
}

async fn get_jwk_set<'t>() -> Result<&'t OnceCell<JWKSet<Empty>>, VerifierError> {
    if JWK_SET.get().is_none() {
        let jwk_set = reqwest::get(&JWKS_URL.jwks_url)
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

    decoded_token.validate(ValidationOptions {
        issuer: Validation::Validate(JWKS_URL.iss.to_owned()),
        ..Default::default()
    })?;
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
    #[error("Unable to verify, not implemented")]
    VerifierNotImplemented,
}

#[derive(Debug, Default)]
pub struct AuthClaims {
    pub user_id: String,
    pub client_id: String,
    pub event_id: String,
    pub scope: String,
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
            client_id,
            event_id,
            scope,
        }
    }
}

impl TryFrom<RequestContext> for AuthClaims {
    type Error = AuthError;

    fn try_from(ctx: RequestContext) -> Result<Self, Self::Error> {
        match ctx {
            RequestContext::ApiGatewayV2(ctx) => {
                let mut jwt = ctx
                    .authorizer
                    .and_then(|authorizer| authorizer.jwt)
                    .ok_or(AuthError::InvalidCredentials)?;
                let auth_claims = {
                    let mut auth_claims = AuthClaims {
                        ..Default::default()
                    };
                    if let Some(username) = jwt.claims.remove("username") {
                        auth_claims.user_id = username;
                    } else {
                        return Err(AuthError::InvalidCredentials);
                    }
                    if let Some(client_id) = jwt.claims.remove("client_id") {
                        auth_claims.client_id = client_id;
                    } else {
                        return Err(AuthError::InvalidCredentials);
                    };
                    if let Some(event_id) = jwt.claims.remove("event_id") {
                        auth_claims.event_id = event_id;
                    } else {
                        return Err(AuthError::InvalidCredentials);
                    };
                    if let Some(scope) = jwt.claims.remove("scope") {
                        auth_claims.scope = scope;
                    } else {
                        return Err(AuthError::InvalidCredentials);
                    };
                    auth_claims
                };
                Ok(auth_claims)
            }
            _ => {
                tracing::error!(message = "lambda request context cannot be verified, verifier not implemented", context = ?ctx);
                Err(AuthError::VerifierNotImplemented)
            }
        }
    }
}

impl AuthClaims {
    async fn try_from<T: AsRef<str>>(t: T) -> Result<Self, AuthError> {
        let tokens = t.as_ref().split(' ').collect::<Vec<&str>>();
        if tokens.len().ne(&2) {
            return Err(AuthError::InvalidCredentials);
        }
        if let ("Bearer", token) = (tokens[0], tokens[1]) {
            let claims = validate_decode_jwt(token).await?;
            Ok(claims.into())
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
}

pub async fn decode_token(token: &str) -> Result<AuthClaims, AuthError> {
    match AuthClaims::try_from(token).await {
        Ok(claims) => Ok(claims),
        Err(err) => {
            tracing::error!(error = ?err, message = "Authentication Error");
            Err(err)
        }
    }
}

pub fn decode_auth_ctx(req_ctx: RequestContext) -> Result<AuthClaims, AuthError> {
    match req_ctx.try_into() {
        Ok(claims) => Ok(claims),
        Err(err) => {
            tracing::error!(error = ?err, message = "Authentication Error");
            Err(err)
        }
    }
}

pub mod warp_filter {
    use super::{decode_auth_ctx, decode_token, AuthClaims};
    use warp::Filter;
    use warp_lambda::lambda_http::request::RequestContext;

    pub fn auth_claims(
    ) -> impl Filter<Extract = (AuthClaims,), Error = warp::Rejection> + Clone + Send + Sync + 'static
    {
        let lambda_auth = warp::any()
            .and(warp::filters::ext::get::<RequestContext>())
            .and_then(|aws_req_context| async move {
                tracing::debug!(message = "lambda request context");
                decode_auth_ctx(aws_req_context).map_err(crate::problem::build)
            });

        let auth = warp::any()
            .and(warp::header::<String>("authorization"))
            .and_then(|token: String| async move {
                tracing::debug!(message = "jwt token authentication");
                decode_token(&token).await.map_err(crate::problem::build)
            });

        lambda_auth.or(auth).unify().map(|auth_claims| {
            tracing::debug!(message = "auth claims intercept", claims = ?auth_claims);
            auth_claims
        })
    }
}
