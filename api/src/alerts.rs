use crate::{
    auth::{self, AuthClaims},
    problem,
    validation::with_validated_json,
};
use dynomite::{
    attr_map,
    dynamodb::{
        DeleteItemError, DeleteItemInput, DynamoDb, DynamoDbClient, GetItemError, GetItemInput,
        PutItemError, PutItemInput,
    },
    retry::{Policy, RetryingDynamoDb},
    AttributeError, FromAttributes as _, Item, Retries,
};
use rusoto_core::RusotoError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::Validate;
use warp::Filter;
use warp_lambda::lambda_http::request::RequestContext;

const TABLE_NAME: &str = "CovinAlerts";

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let lambda_auth = warp::any()
        .and(warp::filters::ext::get::<RequestContext>())
        .and_then(|aws_req_context| async move {
            tracing::debug!(message = "lambda request context");
            auth::decode_auth_ctx(aws_req_context).map_err(problem::build)
        });

    let auth = warp::any()
        .and(warp::header::<String>("authorization"))
        .and_then(|token: String| async move {
            tracing::debug!(message = "jwt token authentication");
            auth::decode_token(&token).await.map_err(problem::build)
        });

    let auth = lambda_auth.or(auth).unify().map(|auth_claims| {
        tracing::debug!(message = "auth claims intercept", claims = ?auth_claims);
        auth_claims
    });

    let retry_policy = Policy::Pause(3, std::time::Duration::from_millis(10));
    let client = DynamoDbClient::new(Default::default()).with_retries(retry_policy);
    let dynamo_db = warp::any().map(move || client.clone());

    let get_alert = warp::get().and(auth).and(dynamo_db.clone()).and_then(
        |AuthClaims { user_id, .. }, dynamo_db: RetryingDynamoDb<DynamoDbClient>| async move {
            let key = attr_map! {
                "user_id" => user_id
            };

            let res: AlertPayload = dynamo_db
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
                })???
                .into();
            Ok::<_, warp::Rejection>(warp::reply::json(&res))
        },
    );

    let create_alert = warp::post()
        .and(warp::path::end())
        .and(auth)
        .and(with_validated_json())
        .and(dynamo_db.clone())
        .and_then(
            |AuthClaims { user_id, .. },
             alert_payload: AlertPayload,
             dynamo_db: RetryingDynamoDb<DynamoDbClient>| async move {
                let alert: Alert = (alert_payload, user_id).into();
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

    let delete_alert = warp::delete().and(auth).and(dynamo_db).and_then(
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

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
struct AlertPayload {
    district_id: u32,
    #[validate(length(min = 1, max = 20))]
    centers: Vec<u32>,
    #[validate(email)]
    email: String,
    #[validate(phone)]
    mobile_no: Option<String>,
    #[validate(range(min = 18))]
    age: u16,
}

#[derive(Debug, Clone, Item)]
pub struct Alert {
    #[dynomite(partition_key)]
    pub user_id: String,
    pub district_id: u32,
    pub centers: Vec<u32>,
    pub email: String,
    pub mobile_no: Option<String>,
    pub age: u16,
}

impl<T: AsRef<str>> From<(AlertPayload, T)> for Alert {
    fn from(
        (
            AlertPayload {
                district_id,
                centers,
                email,
                mobile_no,
                age,
            },
            user_id,
        ): (AlertPayload, T),
    ) -> Self {
        let user_id = user_id.as_ref().to_string();
        Self { user_id, district_id, centers, email, mobile_no, age }
    }
}

impl From<Alert> for AlertPayload {
    fn from(
        Alert {
            district_id,
            centers,
            email,
            mobile_no,
            age,
            ..
        }: Alert,
    ) -> Self {
        Self {
            district_id,
            centers,
            email,
            mobile_no,
            age,
        }
    }
}
