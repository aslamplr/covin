use std::net::SocketAddrV4;

use anyhow::Result;
use covin_backend::{
    common::problem,
    covin::{centers, districts},
};
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{
    self,
    http::{header, Method},
    Filter,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Filter traces based on the RUST_LOG env var, or, if it's not set,
    // default to show the output of the example.
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());
    // Naive check on env:AWS_LAMBDA_RUNTIME_API to have value to see if this is running inside a lambda function
    let is_lambda_env = std::env::var("AWS_LAMBDA_RUNTIME_API").is_ok();

    let tracing_builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE);
    if is_lambda_env {
        tracing_builder.json().init();
    } else {
        tracing_builder.init();
    }

    let cors = warp::cors()
        .allow_methods(&[Method::GET, Method::POST, Method::DELETE])
        .allow_header(header::CONTENT_TYPE)
        .allow_header(header::AUTHORIZATION)
        .allow_any_origin()
        .build();

    let routes = warp::any()
        .and(centers::routes().or(districts::routes()))
        .recover(problem::unpack)
        .with(warp::log("covin::proxy"))
        .with(cors)
        .with(warp::trace::request());

    // To serve warp directly set env WARP_SOCK_ADDR=127.0.0.1:3031
    if !is_lambda_env {
        let addr = std::env::var("WARP_SOCK_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3031".to_string())
            .as_str()
            .parse::<SocketAddrV4>()?;
        warp::serve(routes).run(addr).await;
    } else {
        warp_lambda::run(warp::service(routes)).await?;
    }

    Ok(())
}
