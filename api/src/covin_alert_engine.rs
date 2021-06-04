use anyhow::Error;
use covin_backend::{
    alert_engine::{
        email_client::SesEmailClient, exclusion_map::S3ExclusionMap,
        template_engine::TeraTemplateEngine, AlertEngine,
    },
    api::alerts::AlertService,
    covin::centers::CovinFindCenters,
};
use lamedh_runtime::{handler_fn, run, Context, Error as LambdaError};
use serde_json::{json, Value};
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
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

    if is_lambda_env {
        run(handler_fn(func)).await?;
    } else {
        func(Value::default(), Context::default()).await?;
    }

    Ok(())
}

#[tracing::instrument(level = "debug", err)]
async fn func(_event: Value, _: Context) -> Result<Value, Error> {
    let find_centers = CovinFindCenters::new();
    let exclusion_map = S3ExclusionMap::init().await;
    let ses_client = SesEmailClient::new();
    let tera = TeraTemplateEngine::try_init()?;
    let get_all_alert_configs = || async {
        let alert_service = AlertService::new();
        alert_service.get_all_alert_configs().await
    };
    let mut alert_engine = AlertEngine::new(
        get_all_alert_configs,
        find_centers,
        exclusion_map,
        ses_client,
        tera,
    );
    alert_engine.run().await?;
    Ok(json!({ "message": "Completed!", "status": "ok" }))
}
