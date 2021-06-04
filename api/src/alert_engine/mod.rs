pub mod alert_session;
pub mod email_client;
pub mod exclusion_map;
pub mod template_engine;

use std::collections::HashMap;

use anyhow::Error;
use chrono::{FixedOffset, Utc};

use crate::{
    alert_engine::alert_session::AlertSession,
    api::alerts::{AlertFilter, DoseFilter, GetAlertsError},
    covin::centers::{Center, FindCenters},
};

use self::{
    email_client::EmailClient, exclusion_map::ExclusionMap, template_engine::TemplateEngine,
};

const HOUR: i32 = 3600;

fn get_date_today() -> String {
    let ist_offset = FixedOffset::east(5 * HOUR + HOUR / 2);
    let ist_date_tomorrow = Utc::now() + ist_offset;
    ist_date_tomorrow.format("%d-%m-%Y").to_string()
}

pub struct AlertEngine<GaFn, GaFnFut, Fc, Em, Ec, Te>
where
    GaFn: Fn() -> GaFnFut,
    GaFnFut: futures::Future<Output = Result<Vec<AlertFilter>, GetAlertsError>>,
    Fc: FindCenters,
    Em: ExclusionMap<Key = String, Value = Vec<u32>>,
    Ec: EmailClient,
    Te: TemplateEngine,
{
    exclusion_map: Em,
    template_engine: Te,
    email_client: Ec,
    find_centers: Fc,
    get_alerts: GaFn,
}

impl<GaFn, GaFnFut, Fc, Em, Ec, Te> AlertEngine<GaFn, GaFnFut, Fc, Em, Ec, Te>
where
    GaFn: Fn() -> GaFnFut,
    GaFnFut: futures::Future<Output = Result<Vec<AlertFilter>, GetAlertsError>>,
    Fc: FindCenters,
    Em: ExclusionMap<Key = String, Value = Vec<u32>>,
    Ec: EmailClient,
    Te: TemplateEngine,
{
    pub fn new(
        get_alerts: GaFn,
        find_centers: Fc,
        exclusion_map: Em,
        email_client: Ec,
        template_engine: Te,
    ) -> Self {
        Self {
            exclusion_map,
            template_engine,
            email_client,
            find_centers,
            get_alerts,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn run(&mut self) -> Result<(), Error> {
        let exclusion_map = &mut self.exclusion_map;
        let tera = &self.template_engine;
        let ses_client = &mut self.email_client;
        let find_centers = &self.find_centers;

        let date_today = get_date_today();
        let get_alerts = &self.get_alerts;
        let alerts = get_alerts().await?;

        let grouped = alerts.into_iter().fold(
            HashMap::<u32, Vec<AlertFilter>>::new(),
            |mut grouped, alert| {
                let AlertFilter { district_id, .. } = alert;
                if let Some(vals) = grouped.get_mut(&district_id) {
                    vals.push(alert);
                } else {
                    grouped.insert(district_id, vec![alert]);
                }
                grouped
            },
        );

        for (district_id, alerts) in grouped {
            let res = find_centers
                .get_all_centers_by_district_json(&format!("{}", district_id), &date_today, None)
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
                            let AlertFilter {
                                user_id,
                                centers,
                                age,
                                email,
                                dose,
                                ..
                            } = alert;

                            // TODO: @aslamplr, revert this when ready
                            // let exclude_centers = exclusion_map
                            //     .get(&user_id)
                            //     .map(|x| x.as_slice())
                            //     .unwrap_or_default();

                            // Not respecting exlusion_map for now!!
                            let exclude_centers = &[];
                            let centers_to_check = centers
                                .as_ref()
                                .map(|centers| centers.iter().copied().collect::<Vec<u32>>())
                                .unwrap_or_else(|| {
                                    center_map.keys().into_iter().copied().collect::<Vec<u32>>()
                                });
                            let sessions_to_alert = centers_to_check
                                .iter()
                                .filter(|center_id| !exclude_centers.contains(center_id))
                                .map(|center_id| center_map.get(center_id))
                                .filter(|center| {
                                    center
                                        .map(|center| center.sessions.len().ge(&1))
                                        .unwrap_or(false)
                                })
                                .map(|center| {
                                    center.map(|center| {
                                        center
                                            .sessions
                                            .iter()
                                            .map(|session| AlertSession::from((session, center)))
                                            .filter(|alert_session| {
                                                (match dose {
                                                    DoseFilter::Any => 1_f32.le(&alert_session
                                                        .session
                                                        .available_capacity),
                                                    DoseFilter::First => 1_f32.le(&alert_session
                                                        .session
                                                        .available_capacity_dose1),
                                                    DoseFilter::Second => 1_f32.le(&alert_session
                                                        .session
                                                        .available_capacity_dose2),
                                                }) && age
                                                    .map(|age| {
                                                        age.ge(&alert_session.session.min_age_limit)
                                                    })
                                                    .unwrap_or(true)
                                            })
                                            .collect::<Vec<_>>()
                                    })
                                })
                                .map(|center| center.unwrap())
                                .flatten()
                                .collect::<Vec<AlertSession>>();
                            if !sessions_to_alert.is_empty() {
                                let content = tera.generate_alert_content(&sessions_to_alert)?;
                                tracing::debug!(message = "Found centers for user", %user_id, %email, ?centers, ?sessions_to_alert);
                                ses_client.send_alert_email(&email, &content).await?;
                                exclusion_map.insert(
                                    user_id,
                                    sessions_to_alert
                                        .into_iter()
                                        .map(|session| session.center.center_id)
                                        .collect(),
                                );
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

        exclusion_map.store_exclusion_map().await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, convert::Infallible};

    use async_trait::async_trait;

    use crate::{
        api::alerts::{AlertFilter, DoseFilter, GetAlertsError},
        covin::centers::{Center, CenterResponse, FindCenters, Session},
    };

    use super::{
        alert_session::AlertSession, email_client::EmailClient, exclusion_map::ExclusionMap,
        template_engine::TemplateEngine, AlertEngine,
    };

    // type BoxedStdError = Box<dyn std::error::Error + Send + Sync + 'static>;

    // #[derive(Debug)]
    // struct MockError(BoxedStdError);

    // impl From<BoxedStdError> for MockError {
    //     fn from(boxed_err: BoxedStdError) -> Self {
    //         Self(boxed_err)
    //     }
    // }

    // impl std::fmt::Display for MockError {
    //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //         write!(f, "Mock Error Occured!")
    //     }
    // }

    // impl std::error::Error for MockError {}

    async fn get_mock_alerts() -> Result<Vec<AlertFilter>, GetAlertsError> {
        Ok(vec![
            AlertFilter {
                user_id: "dummy-user-1".to_string(),
                age: Some(18),
                centers: Some(vec![1, 2, 3]),
                district_id: 1,
                dose: DoseFilter::Any,
                email: "dummy-1@email.com".to_string(),
                mobile_no: None,
            },
            AlertFilter {
                user_id: "dummy-user-2".to_string(),
                age: Some(45),
                centers: Some(vec![1, 2, 3]),
                district_id: 1,
                dose: DoseFilter::Any,
                email: "dummy-2@email.com".to_string(),
                mobile_no: None,
            },
            AlertFilter {
                user_id: "dummy-user-3".to_string(),
                age: None,
                centers: None,
                district_id: 1,
                dose: DoseFilter::Any,
                email: "dummy-3@email.com".to_string(),
                mobile_no: None,
            },
            AlertFilter {
                user_id: "dummy-user-4".to_string(),
                age: None,
                centers: None,
                district_id: 1,
                dose: DoseFilter::First,
                email: "dummy-4@email.com".to_string(),
                mobile_no: None,
            },
            AlertFilter {
                user_id: "dummy-user-5".to_string(),
                age: None,
                centers: None,
                district_id: 1,
                dose: DoseFilter::Second,
                email: "dummy-5@email.com".to_string(),
                mobile_no: None,
            },
        ])
    }

    struct MockFindCenters;

    #[async_trait]
    impl FindCenters for MockFindCenters {
        type Error = Infallible;

        async fn get_all_centers_by_district(
            &self,
            _district_id: &str,
            _date: &str,
            _vaccine: Option<&str>,
        ) -> std::result::Result<String, Self::Error> {
            unimplemented!()
        }

        async fn get_all_centers_by_district_json(
            &self,
            _district_id: &str,
            _date: &str,
            _vaccine: Option<&str>,
        ) -> std::result::Result<CenterResponse, Self::Error> {
            Ok(CenterResponse {
                centers: vec![Center {
                    center_id: 1,
                    name: "Dummy Center Name 1".to_string(),
                    sessions: vec![
                        Session {
                            session_id: "dummy-session-id-1".to_string(),
                            min_age_limit: 18,
                            available_capacity: 0_f32,
                            available_capacity_dose1: 0_f32,
                            available_capacity_dose2: 0_f32,
                            ..Default::default()
                        },
                        Session {
                            session_id: "dummy-session-id-2".to_string(),
                            min_age_limit: 45,
                            available_capacity: 0_f32,
                            available_capacity_dose1: 0_f32,
                            available_capacity_dose2: 0_f32,
                            ..Default::default()
                        },
                        Session {
                            session_id: "dummy-session-id-3".to_string(),
                            min_age_limit: 18,
                            available_capacity: 1_f32,
                            available_capacity_dose1: 1_f32,
                            available_capacity_dose2: 0_f32,
                            ..Default::default()
                        },
                        Session {
                            session_id: "dummy-session-id-4".to_string(),
                            min_age_limit: 45,
                            available_capacity: 1_f32,
                            available_capacity_dose1: 1_f32,
                            available_capacity_dose2: 0_f32,
                            ..Default::default()
                        },
                        Session {
                            session_id: "dummy-session-id-5".to_string(),
                            min_age_limit: 18,
                            available_capacity: 1_f32,
                            available_capacity_dose1: 1_f32,
                            available_capacity_dose2: 0_f32,
                            ..Default::default()
                        },
                        Session {
                            session_id: "dummy-session-id-6".to_string(),
                            min_age_limit: 18,
                            available_capacity: 1_f32,
                            available_capacity_dose1: 0_f32,
                            available_capacity_dose2: 1_f32,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
            })
        }
    }

    struct MockExclusionMap(HashMap<String, Vec<u32>>);

    impl MockExclusionMap {
        fn new() -> Self {
            Self(HashMap::new())
        }
    }

    #[async_trait]
    impl ExclusionMap for MockExclusionMap {
        type Key = String;
        type Value = Vec<u32>;
        type Error = Infallible;

        fn _get(&self, _k: &Self::Key) -> Option<&Self::Value> {
            unimplemented!()
        }

        fn insert(&mut self, k: Self::Key, v: Self::Value) -> Option<Self::Value> {
            self.0.insert(k, v)
        }

        async fn store_exclusion_map(&self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    struct MockEmailClient(HashMap<String, String>);

    impl MockEmailClient {
        fn new() -> Self {
            Self(HashMap::new())
        }
    }

    #[async_trait]
    impl EmailClient for MockEmailClient {
        type Error = Infallible;

        async fn send_alert_email(
            &mut self,
            email: &str,
            content: &str,
        ) -> Result<(), Self::Error> {
            self.0.insert(email.to_string(), content.to_string());
            Ok(())
        }
    }

    struct MockTemplateEngine;

    impl TemplateEngine for MockTemplateEngine {
        type Error = Infallible;

        fn generate_alert_content(
            &self,
            sessions_to_alert: &[AlertSession],
        ) -> Result<String, Self::Error> {
            let mut res = String::new();
            sessions_to_alert.iter().for_each(|alert_serssion| {
                use std::fmt::Write;

                let _ = write!(res, "{}\n", alert_serssion);
            });
            Ok(res)
        }
    }

    impl<GaFn, GaFnFut, Fc, Em, Ec, Te> AlertEngine<GaFn, GaFnFut, Fc, Em, Ec, Te>
    where
        GaFn: Fn() -> GaFnFut,
        GaFnFut: futures::Future<Output = Result<Vec<AlertFilter>, GetAlertsError>>,
        Fc: FindCenters,
        Em: ExclusionMap<Key = String, Value = Vec<u32>>,
        Ec: EmailClient,
        Te: TemplateEngine,
    {
        fn get_all_internals(&self) -> (&Em, &Ec) {
            (&self.exclusion_map, &self.email_client)
        }
    }

    #[tokio::test]
    async fn test_alert_engine() {
        let find_centers = MockFindCenters;
        let exclusion_map = MockExclusionMap::new();
        let email_client = MockEmailClient::new();
        let template_engine = MockTemplateEngine;
        let mut alert_engine = AlertEngine::new(
            get_mock_alerts,
            find_centers,
            exclusion_map,
            email_client,
            template_engine,
        );
        let _ = alert_engine.run().await;
        let (_exclusion_map, email_client) = alert_engine.get_all_internals();

        let mut expected_email_map = HashMap::<String, String>::new();
        expected_email_map.insert(
            "dummy-1@email.com".to_string(),
            "dummy-session-id-3\ndummy-session-id-5\ndummy-session-id-6\n".to_string(),
        );
        expected_email_map.insert(
            "dummy-2@email.com".to_string(),
            "dummy-session-id-3\ndummy-session-id-4\ndummy-session-id-5\ndummy-session-id-6\n"
                .to_string(),
        );
        expected_email_map.insert(
            "dummy-3@email.com".to_string(),
            "dummy-session-id-3\ndummy-session-id-4\ndummy-session-id-5\ndummy-session-id-6\n"
                .to_string(),
        );
        expected_email_map.insert(
            "dummy-4@email.com".to_string(),
            "dummy-session-id-3\ndummy-session-id-4\ndummy-session-id-5\n".to_string(),
        );
        expected_email_map.insert(
            "dummy-5@email.com".to_string(),
            "dummy-session-id-6\n".to_string(),
        );

        assert_eq!(email_client.0, expected_email_map);
    }
}
