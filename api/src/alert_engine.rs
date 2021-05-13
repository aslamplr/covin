use anyhow::Error;
use chrono::{Duration, FixedOffset, Utc};
use covin_api::{
    alerts::Alert,
    centers::{get_all_centers_by_district_json, Center},
};
use dynomite::{
    dynamodb::{DynamoDbClient, ScanError, ScanInput},
    retry::Policy,
    AttributeError, DynamoDbExt, Retries,
};
use futures::{future, StreamExt, TryStreamExt};
use lamedh_runtime::{handler_fn, run, Context, Error as LambdaError};
use rusoto_core::RusotoError;
use rusoto_ses::{SendTemplatedEmailError, SendTemplatedEmailRequest, Ses, SesClient};
use serde_json::{json, Value};
use std::{collections::HashMap, convert::TryFrom};
use tera::{Context as TeraContext, Tera};
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

#[tracing::instrument]
fn get_date_tomorrow() -> String {
    let ist_offset = FixedOffset::east(5 * HOUR + HOUR / 2);
    let ist_date_tomorrow = Utc::now() + ist_offset + Duration::days(1);
    ist_date_tomorrow.format("%d-%m-%Y").to_string()
}

#[tracing::instrument]
fn get_tera_template() -> Result<Tera, tera::Error> {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("container", r###"
        {%- for center in centers -%}
            {%- include "available_center" -%}
        {%- endfor -%}
        "###),
        (
            "available_center",
            r###"
<tr style="border-collapse:collapse">
 <td align="left" style="Margin:0;padding-top:5px;padding-bottom:5px;padding-left:40px;padding-right:40px">
  <table width="100%" cellspacing="0" cellpadding="0" style="mso-table-lspace:0pt;mso-table-rspace:0pt;border-collapse:collapse;border-spacing:0px">
   <tr style="border-collapse:collapse">
    <td valign="top" align="center" style="padding:0;Margin:0;width:518px">
     <table style="mso-table-lspace:0pt;mso-table-rspace:0pt;border-collapse:separate;border-spacing:0px;border-left:3px solid #6AA84F;border-right:1px solid #DDDDDD;border-top:1px solid #DDDDDD;border-bottom:1px solid #DDDDDD;background-color:#FFFFFF;border-top-left-radius:2px;border-top-right-radius:2px;border-bottom-right-radius:2px;border-bottom-left-radius:2px" width="100%" cellspacing="0" cellpadding="0" bgcolor="#ffffff" role="presentation">
      <tr style="border-collapse:collapse">
       <td style="padding:0;Margin:0;padding-top:5px;padding-bottom:5px;padding-left:5px">
        <p style="Margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
         {{ center.name }}, {{ center.block_name }}, {{ center.district_name }}, {{ center.pincode }}
        </p>
       </td>
      </tr>
      <tr style="border-collapse:collapse">
       <td style="padding:5px;Margin:0">
        <p style="Margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
            Fee Type: {{ center.fee_type }}
        </p>
        {%- for session in center.sessions -%}
        <hr />
        <p style="Margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
            Date: {{ session.date }}
        </p>
        <p style="Margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
            Available Capacity: {{ session.available_capacity }}
        </p>
        <p style="Margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
            Min Age Limit: {{ session.min_age_limit }}
        </p>
        <p style="Margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
            Slots: {{ session.slots | join(sep = ", ") }}
        </p>
        {%- endfor -%}
       </td>
      </tr></table></td></tr></table></td></tr>"###,
        ),
    ])?;
    Ok(tera)
}

#[tracing::instrument]
fn generate_alert_content(
    tera: &Tera,
    centers_to_alert: &[&Center],
) -> Result<String, tera::Error> {
    let mut tera_context = TeraContext::new();
    tera_context.insert("centers", &centers_to_alert);
    let content = tera.render("container", &tera_context)?;
    Ok(content)
}

#[tracing::instrument]
async fn send_alert_email(
    email: &str,
    content: &str,
) -> Result<(), RusotoError<SendTemplatedEmailError>> {
    let client = SesClient::new(rusoto_core::Region::ApSouth1);

    let _resp = client
        .send_templated_email(SendTemplatedEmailRequest {
            source: "Covin Alert <no-reply+covin-alert@email.covin.app>".to_string(),
            template: "CovinAlert".to_string(),
            destination: rusoto_ses::Destination {
                to_addresses: Some(vec![email.to_string()]),
                bcc_addresses: Some(vec!["covin.alert.no.reply@gmail.com".to_string()]),
                ..Default::default()
            },
            template_data: json!({ "content": content }).to_string(),
            ..Default::default()
        })
        .await?;

    Ok(())
}

#[tracing::instrument]
async fn func(_event: Value, _: Context) -> Result<Value, Error> {
    let tera = get_tera_template()?;
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
                        let centers_to_alert = centers
                            .iter()
                            .map(|center_id| center_map.get(center_id))
                            .filter(|center| {
                                center
                                    .map(|center| {
                                        center.sessions.len().ge(&1)
                                            && center.sessions.iter().any(|session| {
                                                1_f32.le(&session.available_capacity)
                                            })
                                            && center
                                                .sessions
                                                .iter()
                                                .any(|session| age.ge(&session.min_age_limit))
                                    })
                                    .unwrap_or(false)
                            })
                            .map(|center| center.unwrap())
                            .collect::<Vec<&Center>>();
                        if !centers_to_alert.is_empty() {
                            let content = generate_alert_content(&tera, &centers_to_alert)?;
                            tracing::debug!(message = "Found centers for user", %user_id, %email, ?centers, ?centers_to_alert);
                            send_alert_email(&email, &content).await?;
                        } else {
                            tracing::debug!(message = "No centers found for user", %user_id, %email, ?centers);
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
