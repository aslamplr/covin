use anyhow::Error;
use chrono::{Duration, FixedOffset, Utc};
use covin_api::{
    alerts::Alert,
    centers::{get_all_centers_by_district_json, Center, Session},
};
use dynomite::{
    dynamodb::{DynamoDbClient, ScanError, ScanInput},
    retry::Policy,
    AttributeError, DynamoDbExt, Retries,
};
use futures::{future, StreamExt, TryStreamExt};
use lamedh_runtime::{handler_fn, run, Context, Error as LambdaError};
use rusoto_core::RusotoError;
use serde_json::{json, Value};
use std::{collections::HashMap, convert::TryFrom};
use tracing_subscriber::fmt::format::FmtSpan;

const HOUR: i32 = 3600;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
    let is_lambda_env = std::env::var("AWS_LAMBDA_RUNTIME_API").map_or(false, |val| val.ne("true"));

    let tracing_builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE);
    if is_lambda_env {
        tracing_builder.json().init();
    } else {
        tracing_builder.init();
    }

    if is_lambda_env {
        run(handler_fn(func)).await?;
    } else {
        func(Value::default(), Context::default()).await?;
    }

    Ok(())
}

fn get_date_tomorrow() -> String {
    let ist_offset = FixedOffset::east(5 * HOUR + HOUR / 2);
    let ist_date_tomorrow = Utc::now() + ist_offset + Duration::days(1);
    ist_date_tomorrow.format("%d-%m-%Y").to_string()
}

#[tracing::instrument]
async fn func(_event: Value, _: Context) -> Result<Value, Error> {
    let date_tomorrow = get_date_tomorrow();
    let alerts = get_all_alert_configs().await?;
    let grouped =
        alerts
            .into_iter()
            .fold(HashMap::<u32, Vec<Alert>>::new(), |mut grouped, alert| {
                let Alert { district_id, .. } = alert;
                if let Some(vals) = grouped.get_mut(&district_id) {
                    vals.push(alert);
                } else {
                    grouped.insert(district_id, vec![alert]);
                }
                grouped
            });
    for (district_id, alerts) in grouped {
        let res =
            get_all_centers_by_district_json(&format!("{}", district_id), &date_tomorrow, None)
                .await;
        match res {
            Ok(res) => {
                let centers = res.centers;
                if !centers.is_empty() {
                    let center_map = centers
                        .into_iter()
                        .filter(|center| {
                            center
                                .sessions
                                .iter()
                                .any(|session| session.available_capacity >= 1_f32)
                        })
                        .fold(HashMap::<u32, Center>::new(), |mut center_map, center| {
                            let Center { center_id, .. } = center;
                            center_map.insert(center_id, center);
                            center_map
                        });

                    for alert in alerts {
                        let Alert {
                            user_id,
                            centers,
                            age,
                            email,
                            ..
                        } = alert;
                        let centers_message = centers
                        .iter()
                        .map(|center_id| center_map.get(center_id))
                        .filter(|center| {
                            center
                                .map(|center| {
                                    center.sessions.len().ge(&1)
                                        && center
                                            .sessions
                                            .iter()
                                            .any(|session| 1_f32.le(&session.available_capacity))
                                        && center
                                            .sessions
                                            .iter()
                                            .any(|session| age.ge(&session.min_age_limit))
                                })
                                .unwrap_or(false)
                        })
                        .map(|center| center.unwrap())
                        .map(|center| {
                            let Center {
                                name,
                                block_name,
                                district_name,
                                pincode,
                                fee_type,
                                sessions,
                                ..
                            } = center;
                            format!(
                                "{}, {}, {}\nPin: {}\nFee Type: {}\n\nSessions:\n{}\n",
                                name, block_name, district_name, pincode, fee_type, sessions.iter().map(|session: &Session| {
                                    let Session {
                                        date,
                                        available_capacity,
                                        min_age_limit,
                                        slots,
                                        ..
                                    } = session;
                                    format!("Date: {}\nAvailable Capacity: {}\nMinimum Age Limit: {}\nSlots: {}", date, available_capacity, min_age_limit, slots.join(", "))
                                }).collect::<Vec<String>>().join("\n")
                            )
                        })
                        .collect::<Vec<String>>().join("\n");
                        if !centers_message.is_empty() {
                            // Send email from here!
                            println!(
                                "Center Availability for user_id={} {}\n{}",
                                user_id, email, centers_message
                            );
                        } else {
                            tracing::debug!(message = "No centers found for user", %user_id, %email, ?centers)
                        }
                    }
                } else {
                    tracing::debug!(message = "No centers found in district", %district_id);
                }
            }
            Err(err) => {
                tracing::error!(message = "An error occured while calling centers by district api", error = ?err);
            }
        }
    }
    Ok(json!({ "message": "Completed!", "status": "ok" }))
}

#[derive(Debug, thiserror::Error)]
enum EngineError {
    #[error("Rusoto Scan Error")]
    RusotoScanError(#[from] RusotoError<ScanError>),
    #[error("Dynomite Attrute Error")]
    DynomiteAttributeError(#[from] AttributeError),
}

#[tracing::instrument]
async fn get_all_alert_configs() -> Result<Vec<Alert>, EngineError> {
    let retry_policy = Policy::Pause(3, std::time::Duration::from_millis(10));
    let client = DynamoDbClient::new(Default::default()).with_retries(retry_policy);

    client
        .scan_pages(ScanInput {
            table_name: "CovinAlerts".to_string(),
            limit: Some(100),
            ..Default::default()
        })
        .map(|item| item.map(|attrs| Alert::try_from(attrs).map_err(EngineError::from)))
        .filter(|item| future::ready(item.is_ok()))
        .try_collect::<Vec<Result<_, _>>>()
        .await?
        .into_iter()
        .collect::<Result<Vec<Alert>, _>>()
}
