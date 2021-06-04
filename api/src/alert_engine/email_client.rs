use async_trait::async_trait;
use rusoto_core::RusotoError;
use rusoto_ses::{SendTemplatedEmailError, SendTemplatedEmailRequest, Ses, SesClient};
use serde_json::json;

#[async_trait]
pub trait EmailClient {
    type Error: std::error::Error + Sync + Send + 'static;

    async fn send_alert_email(&mut self, email: &str, content: &str) -> Result<(), Self::Error>;
}

pub struct SesEmailClient {
    ses_client: SesClient,
}

impl SesEmailClient {
    pub fn new() -> Self {
        let ses_client = SesClient::new(rusoto_core::Region::ApSouth1);
        Self { ses_client }
    }
}

#[async_trait]
impl EmailClient for SesEmailClient {
    type Error = RusotoError<SendTemplatedEmailError>;

    #[tracing::instrument(level = "debug", skip(self, content))]
    async fn send_alert_email(&mut self, email: &str, content: &str) -> Result<(), Self::Error> {
        let client = &self.ses_client;

        let _resp = client
            .send_templated_email(SendTemplatedEmailRequest {
                source: "Covin Alert <no-reply+covin-alert@email.covin.app>".to_string(),
                template: "CovinAlert".to_string(),
                destination: rusoto_ses::Destination {
                    to_addresses: Some(vec![email.to_string()]),
                    bcc_addresses: Some(vec!["covin.alert.no.reply@gmail.com".to_string()]),
                    ..Default::default()
                },
                template_data: json!({ "content": content }).to_string(),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
