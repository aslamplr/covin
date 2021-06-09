use std::env;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use rusoto_core::RusotoError;
use rusoto_ses::{SendTemplatedEmailError, SendTemplatedEmailRequest, Ses, SesClient};
use serde_json::json;

static CONFIG: Lazy<EmailConfig> = Lazy::new(EmailConfig::init);

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
                source: CONFIG.from_email.clone(),
                template: CONFIG.email_template.clone(),
                destination: rusoto_ses::Destination {
                    to_addresses: Some(vec![email.to_string()]),
                    bcc_addresses: CONFIG.bcc_emails.clone(),
                    ..Default::default()
                },
                template_data: json!({ "content": content }).to_string(),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug)]
struct EmailConfig {
    from_email: String,
    email_template: String,
    bcc_emails: Option<Vec<String>>,
}

impl EmailConfig {
    fn init() -> Self {
        let from_email = env::var("FROM_EMAIL").unwrap();
        let email_template = env::var("EMAIL_TEMPLATE").unwrap();
        let bcc_emails = env::var("BCC_EMAILS")
            .map(|val| {
                val.split(";")
                    .map(|email| email.trim())
                    .map(String::from)
                    .collect::<Vec<String>>()
            })
            .ok();
        Self {
            from_email,
            email_template,
            bcc_emails,
        }
    }
}
