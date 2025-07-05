use std::env;
use eventsource_client as es;
use eventsource_client::Client;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use serde_json::{Value};
use crate::file_utils::update_high_low;

pub static SSE_URL: Lazy<String> = Lazy::new(|| {
    env::var("SSE_URL").expect("Missing SSE_URL env var")
});

pub async fn run_pyth_tracker() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let client = es::ClientBuilder::for_url(SSE_URL.as_str())?.build();
        let mut stream = client.stream();
        while let Some(event_result) = stream.next().await {
            if let Ok(es::SSE::Event(event)) = event_result {
                if let Ok(parsed) = serde_json::from_str::<Value>(&event.data) {
                    if let Some(adjusted_price) = extract_adjusted_price(&parsed) {
                        update_high_low(adjusted_price);
                    } else {
                        eprintln!("⚠ Failed to extract adjusted price from JSON: {:?}", parsed);
                    }
                } else {
                    eprintln!("⚠ Failed to parse incoming JSON data.");
                }
            }
        }
        eprintln!("⚠ SSE stream ended, retrying in 5 seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

fn extract_adjusted_price(json_data: &Value) -> Option<f64> {
    let parsed_array = json_data.get("parsed")?.as_array()?;
    let first_entry = parsed_array.first()?;
    let raw_price_str = first_entry.get("price")?.get("price")?.as_str()?;
    let expo = first_entry.get("price")?.get("expo")?.as_i64()?;
    let raw_price = raw_price_str.parse::<f64>().ok()?;
    Some(raw_price * 10f64.powi(expo as i32))
}