use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;
use warp::Filter;

use crate::problem;

static BASE_URL: Lazy<String> = Lazy::new(|| env::var("BASE_URL").unwrap());

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("centers")
        .and(warp::get())
        .and(warp::path::end())
        .and(warp::query::<CenterQueryParams>())
        .and_then(
            |CenterQueryParams {
                 district_id,
                 date,
                 vaccine,
             }| async move {
                let centers = get_all_centers_by_district(&district_id, &date, vaccine.as_deref())
                    .await
                    .map_err(problem::build)?;
                tracing::info!(
                    target: "covin::proxy",
                    message = "vaccination centers",
                    %date,
                    %district_id,
                    vaccine = vaccine.as_deref().unwrap_or("*"),
                    %centers
                );
                Ok::<_, warp::reject::Rejection>(warp::reply::with_header(
                    centers,
                    "Content-Type",
                    "application/json",
                ))
            },
        )
        .with(warp::trace::named("centers"))
}

async fn get_all_centers_by_district(
    district_id: &str,
    date: &str,
    vaccine: Option<&str>,
) -> Result<String> {
    let query = {
        let mut query = vec![("district_id", district_id), ("date", date)];
        if vaccine.is_some() {
            query.push(("vaccine", vaccine.unwrap()))
        }
        query
    };
    let client = reqwest::Client::new();
    let centers = client
        .get(format!(
            "{}/{}",
            *BASE_URL, "v2/appointment/sessions/calendarByDistrict"
        ))
        .query(&query)
        .send()
        .await?
        .text()
        .await?;
    Ok(centers)
}

#[derive(Debug, Deserialize)]
struct CenterQueryParams {
    district_id: String,
    date: String,
    vaccine: Option<String>,
}
