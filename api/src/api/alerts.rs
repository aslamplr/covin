use crate::common::{
    auth::{warp_filter::auth_claims, AuthClaims},
    problem,
    validation::with_validated_json,
};
pub use service::{AlertError, AlertFilter, DoseFilter};
use service::{AlertPayload, AlertService};
use warp::Filter;

pub fn routes() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let auth = auth_claims();
    let alert_service = AlertService::new();
    let alert_service = warp::any().map(move || alert_service.clone());

    let get_alert = warp::get()
        .and(auth.clone())
        .and(alert_service.clone())
        .and_then(
            |AuthClaims { user_id, .. }, alert_service: AlertService| async move {
                let res = alert_service.get_alert(user_id).await.map_err(build_err)?;
                Ok::<_, warp::Rejection>(warp::reply::json(&res))
            },
        );

    let create_alert = warp::post()
        .and(warp::path::end())
        .and(auth.clone())
        .and(with_validated_json())
        .and(alert_service.clone())
        .and_then(
            |AuthClaims { user_id, .. },
             alert_payload: AlertPayload,
             alert_service: AlertService| async move {
                alert_service
                    .create_alert(alert_payload, &user_id)
                    .await
                    .map_err(build_err)?;
                Ok::<_, warp::Rejection>(warp::reply::with_status(
                    warp::reply::reply(),
                    warp::http::StatusCode::CREATED,
                ))
            },
        );

    let delete_alert = warp::delete().and(auth).and(alert_service).and_then(
        |AuthClaims { user_id, .. }, alert_service: AlertService| async move {
            alert_service
                .delete_alert(user_id)
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

mod service {
    use dynomite::{
        attr_map,
        dynamodb::{
            DeleteItemError, DeleteItemInput, DynamoDb, DynamoDbClient, GetItemError, GetItemInput,
            PutItemError, PutItemInput,
        },
        retry::{Policy, RetryingDynamoDb},
        Attribute, AttributeError, FromAttributes as _, Item, Retries,
    };
    use rusoto_core::RusotoError;
    use serde::{Deserialize, Serialize};
    use thiserror::Error;
    use validator::Validate;

    const TABLE_NAME: &str = "CovinAlerts";

    #[derive(Clone)]
    pub struct AlertService {
        dynamodb_client: RetryingDynamoDb<DynamoDbClient>,
    }

    impl AlertService {
        pub fn new() -> Self {
            let retry_policy = Policy::Pause(3, std::time::Duration::from_millis(10));
            let dynamodb_client =
                DynamoDbClient::new(Default::default()).with_retries(retry_policy);
            Self { dynamodb_client }
        }

        pub async fn get_alert(&self, user_id: String) -> Result<AlertPayload, AlertError> {
            let dynamo_db = &self.dynamodb_client;
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
                .map(|res| {
                    res.item
                        .map(|mut item| AlertFilter::from_attrs(&mut item))
                        .ok_or(AlertError::NothingFound)
                })???
                .into();
            Ok(res)
        }

        pub async fn create_alert(
            &self,
            alert_payload: AlertPayload,
            user_id: &str,
        ) -> Result<(), AlertError> {
            let dynamo_db = &self.dynamodb_client;
            let alert: AlertFilter = (alert_payload, user_id).into();
            dynamo_db
                .put_item(PutItemInput {
                    table_name: TABLE_NAME.to_string(),
                    item: alert.into(),
                    ..PutItemInput::default()
                })
                .await?;

            Ok(())
        }

        pub async fn delete_alert(&self, user_id: String) -> Result<(), AlertError> {
            let dynamo_db = &self.dynamodb_client;
            let key = attr_map! {
                "user_id" => user_id
            };

            dynamo_db
                .delete_item(DeleteItemInput {
                    table_name: TABLE_NAME.to_string(),
                    key,
                    ..DeleteItemInput::default()
                })
                .await?;

            Ok(())
        }
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

    #[derive(Debug, Deserialize, Serialize, Validate, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct AlertPayload {
        pub(crate) district_id: u32,
        #[validate(length(min = 1, max = 20))]
        pub(crate) centers: Option<Vec<u32>>,
        #[validate(email)]
        pub(crate) email: String,
        #[validate(phone)]
        pub(crate) mobile_no: Option<String>,
        #[validate(range(min = 18))]
        pub(crate) age: Option<u16>,
        #[serde(default)]
        pub(crate) dose: DoseFilter,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Attribute)]
    #[serde(rename_all = "camelCase")]
    pub enum DoseFilter {
        Any,
        First,
        Second,
    }

    impl Default for DoseFilter {
        fn default() -> Self {
            Self::Any
        }
    }

    #[derive(Debug, Clone, Item)]
    pub struct AlertFilter {
        #[dynomite(partition_key)]
        pub user_id: String,
        pub district_id: u32,
        pub centers: Option<Vec<u32>>,
        pub email: String,
        pub mobile_no: Option<String>,
        pub age: Option<u16>,
        #[dynomite(default)]
        pub dose: DoseFilter,
    }

    impl<T: AsRef<str>> From<(AlertPayload, T)> for AlertFilter {
        fn from(
            (
                AlertPayload {
                    district_id,
                    centers,
                    email,
                    mobile_no,
                    age,
                    dose,
                },
                user_id,
            ): (AlertPayload, T),
        ) -> Self {
            let user_id = user_id.as_ref().to_string();
            Self {
                user_id,
                district_id,
                centers,
                email,
                mobile_no,
                age,
                dose,
            }
        }
    }

    impl From<AlertFilter> for AlertPayload {
        fn from(
            AlertFilter {
                district_id,
                centers,
                email,
                mobile_no,
                age,
                dose,
                ..
            }: AlertFilter,
        ) -> Self {
            Self {
                district_id,
                centers,
                email,
                mobile_no,
                age,
                dose,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::service::{AlertFilter, AlertPayload, DoseFilter};
    use dynomite::{attr_map, Attributes, FromAttributes as _};
    use serde_json::{from_str, json};

    #[test]
    fn convert_from_json_v1() {
        let json = json!({
            "districtId": 123,
            "centers": [1231, 1232, 1233, 1234],
            "email": "dummy@email.com",
            "mobileNo": "+919123456789",
            "age": 18,
        })
        .to_string();

        let expected_alert_payload = AlertPayload {
            district_id: 123,
            centers: Some(vec![1231, 1232, 1233, 1234]),
            email: "dummy@email.com".to_string(),
            mobile_no: Some("+919123456789".to_string()),
            age: Some(18),
            dose: DoseFilter::Any,
        };

        let alert_payload: AlertPayload = from_str(&json).unwrap();

        assert_eq!(alert_payload, expected_alert_payload);
    }

    #[test]
    fn convert_from_json_v2() {
        let json = json!({
            "districtId": 123,
            "centers": null,
            "email": "dummy@email.com",
            "mobileNo": "+919123456789",
            "age": 18,
            "dose": "any",
        })
        .to_string();

        let expected_alert_payload = AlertPayload {
            district_id: 123,
            centers: None,
            email: "dummy@email.com".to_string(),
            mobile_no: Some("+919123456789".to_string()),
            age: Some(18),
            dose: DoseFilter::Any,
        };

        let alert_payload: AlertPayload = from_str(&json).unwrap();

        assert_eq!(alert_payload, expected_alert_payload);
    }

    #[test]
    fn convert_v2_to_dynamodb_attrs() {
        let alert_payload = AlertPayload {
            district_id: 123,
            centers: None,
            email: "dummy@email.com".to_string(),
            mobile_no: Some("+919123456789".to_string()),
            age: Some(18),
            dose: DoseFilter::Any,
        };

        let alert_filter = AlertFilter::from((alert_payload, "user-id-dummy"));
        let attrs = Attributes::from(alert_filter);

        let expected_attrs = attr_map! {
            "user_id" => "user-id-dummy".to_string(),
            "district_id" => 123,
            "centers" => None::<Vec<u32>>,
            "email" => "dummy@email.com".to_string(),
            "mobile_no" => "+919123456789".to_string(),
            "age" => 18,
            "dose" => "Any".to_string(),
        };
        assert_eq!(attrs, expected_attrs);
    }

    #[test]
    fn convert_from_v1_dynamodb_attrs() {
        let mut attrs = attr_map! {
            "user_id" => "user-id-dummy".to_string(),
            "district_id" => 123,
            "centers" => vec![1231, 1232, 1233],
            "email" => "dummy@email.com".to_string(),
            "mobile_no" => "+919123456789".to_string(),
            "age" => 18,
        };

        let expected_alert_payload = AlertPayload {
            district_id: 123,
            centers: Some(vec![1231, 1232, 1233]),
            email: "dummy@email.com".to_string(),
            mobile_no: Some("+919123456789".to_string()),
            age: Some(18),
            dose: DoseFilter::Any,
        };

        let alert_filter = AlertFilter::from_attrs(&mut attrs).unwrap();
        let alert_payload = AlertPayload::from(alert_filter);

        assert_eq!(alert_payload, expected_alert_payload);
    }

    #[test]
    fn convert_from_v2_dynamodb_attrs() {
        let mut attrs = attr_map! {
            "user_id" => "user-id-dummy".to_string(),
            "district_id" => 123,
            "centers" => None::<Vec<u32>>,
            "email" => "dummy@email.com".to_string(),
            "mobile_no" => None::<Vec<String>>,
            "age" => None::<Vec<u32>>,
            "dose" => "Any".to_string(),
        };

        let expected_alert_payload = AlertPayload {
            district_id: 123,
            centers: None,
            email: "dummy@email.com".to_string(),
            mobile_no: None,
            age: None,
            dose: DoseFilter::Any,
        };

        let alert_filter = AlertFilter::from_attrs(&mut attrs).unwrap();
        let alert_payload = AlertPayload::from(alert_filter);

        assert_eq!(alert_payload, expected_alert_payload);
    }
}
