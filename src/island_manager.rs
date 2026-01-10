use reqwest::Client;
use serde::Deserialize;
use chrono::{DateTime, Utc};
use inquire::Select;

const ISLAND_MAP_URL: &str = "https://app.radiooooo.com/island/map";

#[derive(Debug, Deserialize)]
pub struct Modified {
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct Island {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub modified: Option<Modified>,
}

fn island_modified_timestamp(island: &Island) -> DateTime<Utc> {
    island
        .modified
        .as_ref()
        .and_then(|m| DateTime::parse_from_rfc3339(&m.date).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(DateTime::<Utc>::MIN_UTC)
}

fn sort_islands_by_modified(mut islands: Vec<Island>) -> Vec<Island> {
    islands.sort_by(|a, b| {
        island_modified_timestamp(b).cmp(&island_modified_timestamp(a))
    });
    islands
}

async fn fetch_islands() -> Result<Vec<Island>, reqwest::Error> {
    let client = Client::new();

    client
        .get(ISLAND_MAP_URL)
        .send()
        .await?
        .json::<Vec<Island>>()
        .await
}

fn island_labels(islands: &[Island]) -> Vec<String> {
    islands
        .iter()
        .map(|i| {
            let category = i.category.as_deref().unwrap_or("other");
            format!("{} [{}]", i.name, category)
        })
        .collect()
}

pub async fn select_island() -> Island {
    let islands = fetch_islands()
        .await
        .expect("Failed to fetch islands");

    let islands = sort_islands_by_modified(islands);
    let labels = island_labels(&islands);

    let selected_label = Select::new(
        "Select an island:",
        labels,
    )
    .prompt()
    .expect("Selection aborted");

    // Map label → Island
    islands
        .into_iter()
        .find(|i| selected_label.starts_with(&i.name))
        .expect("Selected island not found")
}
