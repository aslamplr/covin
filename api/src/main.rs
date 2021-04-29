mod problem;

use std::env;
use anyhow::Result;
use serde::Deserialize;
use warp::Filter;
use once_cell::sync::Lazy;

static BASE_URL: Lazy<String> = Lazy::new(|| env::var("BASE_URL").unwrap());
static DISTRICTS_URL: Lazy<String> = Lazy::new(|| env::var("DISTRICTS_URL").unwrap());
static BEARER_TOKEN: Lazy<String> = Lazy::new(|| env::var("BEARER_TOKEN").unwrap());

#[tokio::main]
async fn main() -> Result<()> {
    let districts = warp::path("districts")
        .and(warp::get())
        .and(warp::path::end())
        .and_then(|| async {
            let districts = get_all_districts().await.map_err(problem::build)?;
            Ok::<_, warp::reject::Rejection>(warp::reply::with_header(
                districts,
                "Content-Type",
                "application/json",
            ))
        });

    let centers = warp::path("centers")
        .and(warp::get())
        .and(warp::path::end())
        .and(warp::query::<CenterQueryParams>())
        .and_then(
            |CenterQueryParams {
                 district_id,
                 date,
                 vaccine,
             }| async move {
                let centers = get_all_centers_by_district(&district_id, &date, &vaccine)
                    .await
                    .map_err(problem::build)?;
                println!("{}", centers);
                Ok::<_, warp::reject::Rejection>(warp::reply::with_header(
                    centers,
                    "Content-Type",
                    "application/json",
                ))
            },
        );

    let cors = warp::cors()
        .allow_methods(vec!["GET"])
        .allow_any_origin()
        .build();

    let cors_route = warp::options()
        .map(warp::reply)
        .with(cors.clone());

    let routes = warp::path("api")
        .and(districts.or(centers).or(cors_route).with(cors))
        .recover(problem::unpack)
        .with(warp::log("covin::proxy"));

    warp_lambda::run(warp::service(routes)).await?;

    Ok(())
}

async fn get_all_districts() -> Result<String> {
    let districts = reqwest::get(&*DISTRICTS_URL)
        .await?
        .text()
        .await?;
    Ok(districts)
}

async fn get_all_centers_by_district(
    district_id: &str,
    date: &str,
    vaccine: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let centers = client
        .get(format!(
            "{}/{}",
            *BASE_URL, "v2/appointment/sessions/calendarByDistrict"
        ))
        .bearer_auth(&*BEARER_TOKEN)
        .query(&[
            ("district_id", district_id),
            ("date", date),
            ("vaccine", vaccine),
        ])
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
    vaccine: String,
}
