use crate::{auth::{self, AuthClaims, AuthError}, problem};
use dynomite::{
    attr_map,
    dynamodb::{
        DeleteItemError, DeleteItemInput, DynamoDb, DynamoDbClient, GetItemError, GetItemInput,
        PutItemError, PutItemInput,
    },
    retry::{Policy, RetryingDynamoDb},
    AttributeError, Attributes, FromAttributes as _, Item, Retries,
};
use rusoto_core::RusotoError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use warp::Filter;
use warp_lambda::lambda_http::request::RequestContext;
use serde_json::Value;

const TABLE_NAME: &str = "CovinAlerts";

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let lambda_auth = warp::any().and(warp::filters::ext::get::<RequestContext>()).and_then(|aws_req_context| async move {
        let claims = match aws_req_context {
            RequestContext::ApiGatewayV1(ctx) => {
                let mut authorizer = ctx.authorizer;
                let mut claims = authorizer
                    .remove("claims")
                    .ok_or_else(warp::reject::reject)?;
                if let Value::String(cognito_username) = claims["cognito:username"].take() {
                    Ok(AuthClaims {
                        user_id: cognito_username,
                        ..Default::default()
                    })
                } else {
                    Err(AuthError::InvalidCredentials)
                }
            }
            RequestContext::ApiGatewayV2(ctx) => {
                let authorizer = ctx.authorizer.ok_or_else(warp::reject::reject)?;
                let mut jwt = authorizer.jwt.ok_or_else(warp::reject::reject)?;
                if let Some(cognito_username) = jwt.claims.remove("cognito:username") {
                    Ok(AuthClaims {
                        user_id: cognito_username,
                        ..Default::default()
                    })
                } else {
                    Err(AuthError::InvalidCredentials)
                }
            }
            _ => Err(AuthError::InvalidCredentials),
        };
        claims.map_err(problem::build)
    });

    let auth = warp::any().and(warp::header::<String>("authorization")).and_then(|token: String| async move {
        match auth::decode_token(&token).await {
            Ok(auth_claims) => Ok(auth_claims),
            Err(err) => Err(problem::build(err)),
        }
    });

    let auth = lambda_auth.or(auth).unify();

    let retry_policy = Policy::Pause(3, std::time::Duration::from_millis(10));
    let client = DynamoDbClient::new(Default::default()).with_retries(retry_policy);
    let dynamo_db = warp::any().map(move || client.clone());

    let get_alert = warp::get()
        .and(auth)
        .and(dynamo_db.clone())
        .and_then(
            |AuthClaims { user_id, .. }, dynamo_db: RetryingDynamoDb<DynamoDbClient>| async move {
                let key = attr_map! {
                    "user_id" => user_id
                };

                let res = dynamo_db
                    .get_item(GetItemInput {
                        table_name: TABLE_NAME.to_string(),
                        key,
                        ..GetItemInput::default()
                    })
                    .await
                    .map_err(build_err)
                    .map(|res| {
                        res.item
                            .map(|mut item| Alert::from_attrs(&mut item).map_err(build_err))
                            .ok_or(AlertError::NothingFound)
                            .map_err(build_err)
                    })???;
                Ok::<_, warp::Rejection>(warp::reply::json(&res))
            },
        );

    let create_alert = warp::post()
        .and(warp::path::end())
        .and(auth)
        .and(warp::body::json())
        .and(dynamo_db.clone())
        .and_then(
            |AuthClaims { user_id, .. }, alert_payload: AlertPayload, dynamo_db: RetryingDynamoDb<DynamoDbClient>| async move {
                let mut alert: Alert = alert_payload.into();
                alert.user_id = user_id;
                dynamo_db
                    .put_item(PutItemInput {
                        table_name: TABLE_NAME.to_string(),
                        item: alert.into(),
                        ..PutItemInput::default()
                    })
                    .await
                    .map_err(build_err)?;
                Ok::<_, warp::Rejection>(warp::reply::with_status(
                    warp::reply::reply(),
                    warp::http::StatusCode::CREATED,
                ))
            },
        );

    let delete_alert = warp::delete()
        .and(auth)
        .and(dynamo_db)
        .and_then(
            |AuthClaims { user_id, .. }, dynamo_db: RetryingDynamoDb<DynamoDbClient>| async move {
                let key = attr_map! {
                    "user_id" => user_id
                };

                dynamo_db
                    .delete_item(DeleteItemInput {
                        table_name: TABLE_NAME.to_string(),
                        key,
                        ..DeleteItemInput::default()
                    })
                    .await
                    .map_err(build_err)?;
                Ok::<_, warp::Rejection>(warp::reply::with_status(
                    warp::reply::reply(),
                    warp::http::StatusCode::NO_CONTENT,
                ))
            },
        );

    warp::path!("alerts" / "register" / ..)
        .and(get_alert.or(create_alert).or(delete_alert))
        .with(warp::trace::named("alerts"))
}

fn build_err<E: Into<AlertError>>(err: E) -> warp::Rejection {
    problem::build(err.into())
}

#[derive(Debug, Error)]
pub enum AlertError {
    #[error("unable to create alert")]
    UnableToCreate(#[from] RusotoError<PutItemError>),
    #[error("unable to get alert")]
    UnableToGet(#[from] RusotoError<GetItemError>),
    #[error("unable to parse attributes")]
    UnableToParseAttr(#[from] AttributeError),
    #[error("unable to find alert")]
    NothingFound,
    #[error("unable to get alert")]
    UnableToDelete(#[from] RusotoError<DeleteItemError>),
}

#[derive(Debug, Deserialize)]
struct AlertPayload {
    location: Location,
    district_id: u32,
    email: String,
    mobile_no: String,
    age: u16,
    year_of_birth: u16,
    kilometers: u32,
}

#[derive(Debug, Clone, Serialize, Item)]
struct Alert {
    #[dynomite(partition_key)]
    user_id: String,
    location: Location,
    district_id: u32,
    email: String,
    mobile_no: String,
    age: u16,
    year_of_birth: u16,
    kilometers: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Attributes)]
struct Location {
    lat: f32,
    long: f32,
}

impl From<AlertPayload> for Alert {
    fn from(
        AlertPayload {
            location,
            district_id,
            email,
            mobile_no,
            age,
            year_of_birth,
            kilometers,
        }: AlertPayload,
    ) -> Self {
        Self {
            user_id: "abcdefgh".into(),
            location: Location { ..location },
            district_id,
            email,
            mobile_no,
            age,
            year_of_birth,
            kilometers,
        }
    }
}
