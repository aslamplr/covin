use anyhow::Result;
use once_cell::sync::Lazy;
use std::env;
use warp::Filter;

use crate::problem;

static DISTRICTS_URL: Lazy<String> = Lazy::new(|| env::var("DISTRICTS_URL").unwrap());

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("districts")
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
        .with(warp::trace::named("districts"))
}

async fn get_all_districts() -> Result<String> {
    let districts = reqwest::get(&*DISTRICTS_URL).await?.text().await?;
    Ok(districts)
}
