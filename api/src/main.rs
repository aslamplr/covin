use anyhow::Result;
use covin_api::{alerts, centers, districts, problem};
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<()> {
    // Filter traces based on the RUST_LOG env var, or, if it's not set,
    // default to show the output of the example.
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let cors = warp::cors()
        .allow_methods(vec!["GET"])
        .allow_any_origin()
        .build();

    let routes = warp::path("api")
        .and(
            centers::routes()
                .or(districts::routes())
                .or(alerts::routes()),
        )
        .recover(problem::unpack)
        .with(warp::log("covin::proxy"))
        .with(cors)
        .with(warp::trace::request());

    // To serve locally uncomment the following
    // warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;

    warp_lambda::run(warp::service(routes)).await?;

    Ok(())
}
