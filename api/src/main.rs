mod problem;

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

static BASE_URL: Lazy<String> = Lazy::new(|| env::var("BASE_URL").unwrap());
static DISTRICTS_URL: Lazy<String> = Lazy::new(|| env::var("DISTRICTS_URL").unwrap());

#[tokio::main]
async fn main() -> Result<()> {
    // Filter traces based on the RUST_LOG env var, or, if it's not set,
    // default to show the output of the example.
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

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
        })
        .with(warp::trace::named("districts"));

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
                let centers = get_all_centers_by_district(&district_id, &date, vaccine.as_deref())
                    .await
                    .map_err(problem::build)?;
                tracing::info!(
                    "centers: date={}; district_id={}; vaccine={}; \n{}",
                    date,
                    district_id,
                    vaccine.as_deref().unwrap_or_else(|| "*"),
                    centers
                );
                Ok::<_, warp::reject::Rejection>(warp::reply::with_header(
                    centers,
                    "Content-Type",
                    "application/json",
                ))
            },
        )
        .with(warp::trace::named("centers"));

    let cors = warp::cors()
        .allow_methods(vec!["GET"])
        .allow_any_origin()
        .build();

    let routes = warp::path("api")
        .and(districts.or(centers))
        .recover(problem::unpack)
        .with(warp::log("covin::proxy"))
        .with(cors)
        .with(warp::trace::request());

    // To serve locally uncomment the following
    // warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;

    warp_lambda::run(warp::service(routes)).await?;

    Ok(())
}

async fn get_all_districts() -> Result<String> {
    let districts = reqwest::get(&*DISTRICTS_URL).await?.text().await?;
    Ok(districts)
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
