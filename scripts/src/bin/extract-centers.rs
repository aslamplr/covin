use anyhow::Result;
use futures::future;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static REPORT_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("REPORT_URL").unwrap_or_else(|_| {
        "https://api.cowin.gov.in/api/v1/reports/v2/getPublicReports".to_string()
    })
});
const DISTRICT_IDS: [u16; 14] = [
    301, // "Alappuzha"
    307, // "Ernakulam"
    306, // "Idukki"
    297, // "Kannur"
    295, // "Kasargod"
    298, // "Kollam"
    304, // "Kottayam"
    305, // "Kozhikode"
    302, // "Malappuram"
    308, // "Palakkad"
    300, // "Pathanamthitta"
    296, // "Thiruvananthapuram"
    303, // "Thrissur"
    299, // "Wayanad"
];
const STATE_ID_KL: u16 = 17;

#[tokio::main]
async fn main() -> Result<()> {
    let centers = future::try_join_all(
        DISTRICT_IDS
            .iter()
            .map(|district_id| get_all_centers(STATE_ID_KL, *district_id)),
    )
    .await?
    .into_iter()
    .flatten()
    .collect::<Vec<Center>>();
    save_as_json("./all_centers.json", &centers)?;
    Ok(())
}

fn save_as_json<T: AsRef<std::path::Path>, V: Serialize>(file_name: T, centers: &V) -> Result<()> {
    let file_name: &std::path::Path = file_name.as_ref();
    let json = serde_json::to_string(centers)?;
    std::fs::write(file_name, json)?;
    Ok(())
}

async fn get_all_centers(state_id: u16, district_id: u16) -> Result<Vec<Center>> {
    let Report { sessions } = reqwest::get(format!(
        "{}?state_id={}&district_id={}",
        &*REPORT_URL, state_id, district_id,
    ))
    .await?
    .json::<Report>()
    .await?;

    let centers = sessions
        .into_iter()
        .map(|session| Center::new(session, state_id, district_id))
        .collect::<Vec<Center>>();

    Ok(centers)
}

#[derive(Debug, Deserialize)]
struct Report {
    #[serde(rename = "getBeneficiariesGroupBy")]
    sessions: Vec<Session>,
}

#[derive(Debug, Deserialize)]
struct Session {
    #[serde(rename = "session_site_id")]
    center_id: u32,
    #[serde(rename = "session_site_name")]
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Center {
    center_id: u32,
    name: String,
    district_id: u16,
    state_id: u16,
}

impl Center {
    fn new(Session { center_id, name }: Session, state_id: u16, district_id: u16) -> Self {
        Self {
            center_id,
            name,
            district_id,
            state_id,
        }
    }
}
