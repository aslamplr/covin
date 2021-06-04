use std::collections::HashMap;

use async_trait::async_trait;
use futures::TryStreamExt;
use rusoto_core::RusotoError;
use rusoto_s3::{GetObjectRequest, PutObjectError, PutObjectRequest, S3Client, S3};

const EXCLUSION_MAP_S3_BUCKET: &str = "covin-transactions";
const EXCLUSION_MAP_S3_KEY: &str = "exclusion_map.json";

#[async_trait]
pub trait ExclusionMap {
    type Key;
    type Value;
    type Error: std::error::Error + Sync + Send + 'static;

    fn _get(&self, k: &Self::Key) -> Option<&Self::Value>;
    fn insert(&mut self, k: Self::Key, v: Self::Value) -> Option<Self::Value>;
    async fn store_exclusion_map(&self) -> Result<(), Self::Error>;
}

pub struct S3ExclusionMap {
    s3_client: S3Client,
    initial_content_length: usize,
    exclusion_map: HashMap<String, Vec<u32>>,
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
    pub async fn init_exclusion_map(s3_client: &S3Client) -> (HashMap<String, Vec<u32>>, usize) {
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
                let value: HashMap<String, Vec<u32>> =
                    serde_json::from_slice(&body).unwrap_or_default();
                tracing::debug!(message = "exclusion map", ?value);
                return (value, content_length);
            }
        }
        (HashMap::<String, Vec<u32>>::new(), 0)
    }
}

#[async_trait]
impl ExclusionMap for S3ExclusionMap {
    type Key = String;
    type Value = Vec<u32>;
    type Error = RusotoError<PutObjectError>;

    fn _get(&self, k: &Self::Key) -> Option<&Self::Value> {
        self.exclusion_map.get(k)
    }

    fn insert(&mut self, k: Self::Key, v: Self::Value) -> Option<Self::Value> {
        self.exclusion_map.insert(k, v)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn store_exclusion_map(&self) -> Result<(), Self::Error> {
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
