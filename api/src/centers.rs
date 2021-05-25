use crate::problem;
use serde::Deserialize;
pub use service::{Center, FindCenters};
use warp::Filter;

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
                let find_centers = FindCenters::new();
                let centers = find_centers
                    .get_all_centers_by_district(&district_id, &date, vaccine.as_deref())
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

#[derive(Debug, Deserialize)]
struct CenterQueryParams {
    pub district_id: String,
    pub date: String,
    pub vaccine: Option<String>,
}
mod service {
    use anyhow::Result;
    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};
    use std::env;

    static CONFIG: Lazy<CentersConfig> = Lazy::new(CentersConfig::init);

    #[derive(Default)]
    pub struct FindCenters {
        client: reqwest::Client,
    }

    impl FindCenters {
        pub fn new() -> Self {
            let headers = {
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::USER_AGENT,
                    CONFIG.user_agent_header.parse().unwrap(),
                );
                headers.insert(
                    reqwest::header::REFERER,
                    CONFIG.referer_header.parse().unwrap(),
                );
                headers.insert(
                    reqwest::header::ORIGIN,
                    CONFIG.origin_header.parse().unwrap(),
                );
                headers
            };
            let client = reqwest::ClientBuilder::new()
                .default_headers(headers)
                .build()
                .unwrap_or_default();
            Self { client }
        }

        async fn get_all_centers_by_district_base(
            &self,
            district_id: &str,
            date: &str,
            vaccine: Option<&str>,
        ) -> Result<reqwest::Response> {
            let client = &self.client;
            let query = {
                let mut query = vec![("district_id", district_id), ("date", date)];
                if vaccine.is_some() {
                    query.push(("vaccine", vaccine.unwrap()))
                }
                query
            };
            Ok(client
                .get(format!(
                    "{}/{}",
                    CONFIG.base_url, "v2/appointment/sessions/calendarByDistrict"
                ))
                .query(&query)
                .send()
                .await
                .and_then(|resp| {
                    let status = resp.status();
                    if status.is_success() || status.is_redirection() {
                        Ok(resp)
                    } else {
                        resp.error_for_status()
                    }
                })?)
        }

        #[tracing::instrument(skip(self))]
        pub async fn get_all_centers_by_district_json(
            &self,
            district_id: &str,
            date: &str,
            vaccine: Option<&str>,
        ) -> Result<CenterResponse> {
            Ok(self
                .get_all_centers_by_district_base(district_id, date, vaccine)
                .await?
                .json()
                .await?)
        }

        #[tracing::instrument(skip(self))]
        pub async fn get_all_centers_by_district(
            &self,
            district_id: &str,
            date: &str,
            vaccine: Option<&str>,
        ) -> Result<String> {
            Ok(self
                .get_all_centers_by_district_base(district_id, date, vaccine)
                .await?
                .text()
                .await?)
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CenterResponse {
        pub centers: Vec<Center>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Center {
        pub center_id: u32,
        pub name: String,
        pub state_name: String,
        pub district_name: String,
        pub block_name: String,
        pub pincode: u32,
        pub from: String,
        pub to: String,
        pub lat: f32,
        pub long: f32,
        pub fee_type: String,
        pub sessions: Vec<Session>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Session {
        pub session_id: String,
        pub available_capacity: f32,
        pub min_age_limit: u16,
        pub date: String,
        pub slots: Vec<String>,
    }

    #[derive(Debug)]
    struct CentersConfig {
        pub base_url: String,
        pub user_agent_header: String,
        pub referer_header: String,
        pub origin_header: String,
    }

    impl CentersConfig {
        fn init() -> Self {
            let base_url = env::var("BASE_URL").unwrap();
            let user_agent_header = env::var("USER_AGENT_HEADER").unwrap();
            let referer_header = env::var("REFERER_HEADER").unwrap();
            let origin_header = env::var("ORIGIN_HEADER").unwrap();
            Self {
                base_url,
                user_agent_header,
                referer_header,
                origin_header,
            }
        }
    }
}
