use crate::problem;
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

const TABLE_NAME: &str = "CovinAlerts";

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let retry_policy = Policy::Pause(3, std::time::Duration::from_millis(10));
    let client = DynamoDbClient::new(Default::default()).with_retries(retry_policy);
    let dynamo_db = warp::any().map(move || client.clone());

    let get_alert = warp::path!(String)
        .and(warp::get())
        .and(dynamo_db.clone())
        .and_then(
            |_user_id, dynamo_db: RetryingDynamoDb<DynamoDbClient>| async move {
                let key = attr_map! {
                    "user_id" => "abcdefgh".to_string()
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
        .and(warp::body::json())
        .and(dynamo_db.clone())
        .and_then(
            |alert_payload: AlertPayload, dynamo_db: RetryingDynamoDb<DynamoDbClient>| async move {
                let alert: Alert = alert_payload.into();
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

    let delete_alert = warp::path!(String)
        .and(warp::delete())
        .and(dynamo_db)
        .and_then(
            |_user_id, dynamo_db: RetryingDynamoDb<DynamoDbClient>| async move {
                let key = attr_map! {
                    "user_id" => "abcdefgh".to_string()
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
