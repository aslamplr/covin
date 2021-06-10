use std::collections::HashMap;

use async_trait::async_trait;
use futures::TryStreamExt;
use rusoto_core::RusotoError;
use rusoto_s3::{GetObjectRequest, PutObjectError, PutObjectRequest, S3Client, S3};

use super::alert_session::AlertSession;

const EXCLUSION_MAP_S3_BUCKET: &str = "covin-transactions";
const EXCLUSION_MAP_S3_KEY: &str = "exclusion_map.json";

#[async_trait]
pub trait ExclusionMap {
    type Error: std::error::Error + Sync + Send + 'static;

    fn any_variance(&self, user_id: &str, session_id: &str, capacity: f32) -> bool;
    fn add(&mut self, user_id: &str, sessions: &[AlertSession]);
    async fn store(&self) -> Result<(), Self::Error>;
}

pub struct S3ExclusionMap {
    s3_client: S3Client,
    initial_content_length: usize,
    exclusion_map: HashMap<String, Vec<(String, f32)>>,
}

impl S3ExclusionMap {
    pub async fn init() -> Self {
        let s3_client = S3Client::new(rusoto_core::Region::ApSouth1);
        let (exclusion_map, content_length) = Self::init_exclusion_map(&s3_client).await;
        Self {
            s3_client,
            exclusion_map,
            initial_content_length: content_length,
        }
    }

    #[tracing::instrument(level = "debug", skip(s3_client))]
    pub async fn init_exclusion_map(
        s3_client: &S3Client,
    ) -> (HashMap<String, Vec<(String, f32)>>, usize) {
        if let Ok(resp) = s3_client
            .get_object(GetObjectRequest {
                bucket: EXCLUSION_MAP_S3_BUCKET.to_string(),
                key: EXCLUSION_MAP_S3_KEY.to_string(),
                ..Default::default()
            })
            .await
        {
            if let Some(body) = resp.body {
                let body = body
                    .map_ok(|b| b.to_vec())
                    .try_concat()
                    .await
                    .unwrap_or_default();
                let content_length = body.len();
                let value: HashMap<String, Vec<(String, f32)>> =
                    serde_json::from_slice(&body).unwrap_or_default();
                tracing::debug!(message = "exclusion map", ?value);
                return (value, content_length);
            }
        }
        (HashMap::<String, Vec<(String, f32)>>::new(), 0)
    }
}

#[async_trait]
impl ExclusionMap for S3ExclusionMap {
    type Error = RusotoError<PutObjectError>;

    fn add(&mut self, user_id: &str, sessions: &[AlertSession]) {
        let vals = sessions
            .iter()
            .map(|session| {
                (
                    session.session.session_id.to_owned(),
                    session.session.available_capacity,
                )
            })
            .collect::<Vec<_>>();
        if let Some(existing_vals) = self.exclusion_map.get_mut(user_id) {
            vals.into_iter().for_each(|(session_id, capacity)| {
                if let Some(mut exist_val) = existing_vals
                    .iter_mut()
                    .find(|(s_id, _cap)| s_id == &session_id)
                {
                    exist_val.1 = capacity;
                } else {
                    existing_vals.push((session_id, capacity));
                }
            });
        } else {
            self.exclusion_map.insert(user_id.to_owned(), vals);
        }
    }

    fn any_variance(&self, user_id: &str, session_id: &str, capacity: f32) -> bool {
        let sessions_for_user = self
            .exclusion_map
            .get(user_id)
            .map(|vals| vals.as_slice())
            .unwrap_or_else(|| &[]);
        sessions_for_user
            .iter()
            .find(|(s_id, _cap)| s_id == session_id)
            .map(|(_s_id, cap)| cap > &capacity || cap < &capacity)
            .unwrap_or(true)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn store(&self) -> Result<(), Self::Error> {
        let s3_client = &self.s3_client;
        let exclusion_map = &self.exclusion_map;
        let initial_content_length = self.initial_content_length;
        let json = serde_json::to_string(exclusion_map)?.as_bytes().to_vec();
        if initial_content_length != json.len() {
            let _resp = s3_client
                .put_object(PutObjectRequest {
                    bucket: EXCLUSION_MAP_S3_BUCKET.to_string(),
                    key: EXCLUSION_MAP_S3_KEY.to_string(),
                    body: Some(json.into()),
                    content_type: Some("appliaction/json".to_string()),
                    ..Default::default()
                })
                .await?;
            tracing::debug!(message = "Stored the exclusion_map")
        } else {
            tracing::debug!(
                message = "No change in content_length, not storing the exclusion_map",
                initial_content_length,
                current_content_length = json.len(),
                ?exclusion_map
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        alert_engine::alert_session::AlertSession,
        covin::centers::{Center, Session},
    };

    use super::{ExclusionMap, S3Client, S3ExclusionMap};

    #[test]
    fn test_exclusion_map() {
        let mut exclusion_map = S3ExclusionMap {
            exclusion_map: HashMap::<String, Vec<(String, f32)>>::new(),
            initial_content_length: 0,
            s3_client: S3Client::new(rusoto_core::Region::ApSouth1),
        };
        let user_id = "some-user-id";

        // Variance should be true when
        // the session id doesn't already exists for user in exclusion_map
        assert_eq!(
            exclusion_map.any_variance(user_id, "session-id-1", 1_f32),
            true
        );

        let center = Center {
            ..Default::default()
        };

        let session = Session {
            session_id: "session-id-1".to_string(),
            available_capacity: 1_f32,
            ..Default::default()
        };
        let sessions_1 = vec![AlertSession {
            center: &center,
            session: &session,
        }];
        exclusion_map.add(user_id, &sessions_1);

        // Variance should be false, when
        // the session id exists in the exclusion_map and no change in the capacity
        assert_eq!(
            exclusion_map.any_variance(user_id, "session-id-1", 1_f32),
            false
        );
        let session = Session {
            session_id: "session-id-1".to_string(),
            available_capacity: 5_f32,
            ..Default::default()
        };
        let sessions_2 = vec![AlertSession {
            center: &center,
            session: &session,
        }];
        exclusion_map.add(user_id, &sessions_2);

        // Variance should be true, when there is change in capacity
        assert_eq!(
            exclusion_map.any_variance(user_id, "session-id-1", 1_f32),
            true
        );

        let session = Session {
            session_id: "session-id-2".to_string(),
            available_capacity: 5_f32,
            ..Default::default()
        };
        let sessions_3 = vec![AlertSession {
            center: &center,
            session: &session,
        }];
        exclusion_map.add(user_id, &sessions_3);

        // Variance should be false, when there is new session added
        // and previously added session should remain for the user!
        assert_eq!(
            exclusion_map.any_variance(user_id, "session-id-1", 5_f32),
            false
        );

        // Newly added session should also remain
        assert_eq!(
            exclusion_map.any_variance(user_id, "session-id-2", 5_f32),
            false
        );
    }
}
