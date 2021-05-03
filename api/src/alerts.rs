use crate::problem;
use dynomite::{
    attr_map,
    dynamodb::{DynamoDb, DynamoDbClient, GetItemInput, PutItemInput},
    retry::{Policy, RetryingDynamoDb},
    Attributes, FromAttributes as _, Item, Retries,
};
use serde::{Deserialize, Serialize};
use warp::Filter;

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
                        table_name: "CovinAlerts".to_string(),
                        key,
                        ..GetItemInput::default()
                    })
                    .await
                    .ok()
                    .map(|res| res.item)
                    .flatten()
                    .map(|mut item| Alert::from_attrs(&mut item).ok())
                    .flatten()
                    .ok_or(anyhow::anyhow!("Unable to fetch alert for user!"))
                    .map_err(problem::build)?;
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
                        table_name: "CovinAlerts".to_string(),
                        item: alert.into(),
                        ..PutItemInput::default()
                    })
                    .await
                    .map_err(problem::build)?;
                Ok::<_, warp::Rejection>(warp::reply::with_status(
                    warp::reply::reply(),
                    warp::http::StatusCode::CREATED,
                ))
            },
        );

    let update_alert = warp::path!(String)
        .and(warp::put())
        .and(warp::body::json::<AlertPayload>())
        .and(dynamo_db.clone())
        .map(|user_id, alert_req, _dynamo_db| {
            format!("UPDATE Alert of user {} \n{:#?}", user_id, alert_req)
        });

    let delete_alert = warp::path!(String)
        .and(warp::delete())
        .and(dynamo_db)
        .map(|user_id, _dynamo_db| format!("DELETE Alert of user {}", user_id));

    warp::path!("alerts" / "register" / ..)
        .and(get_alert.or(create_alert).or(update_alert).or(delete_alert))
        .with(warp::trace::named("alerts"))
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
