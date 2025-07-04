use eventsource_client as es;
use eventsource_client::Client;
use futures_util::StreamExt;
use serde_json::{Value};
use crate::config::SSE_URL;
use crate::file_utils::update_high_low;


pub async fn run_pyth_tracker() -> Result<(), Box<dyn std::error::Error>> {
    let client = es::ClientBuilder::for_url(SSE_URL.as_str())?
        .build();
    let mut stream = client.stream();

    while let Some(event_result) = stream.next().await {
        if let Ok(es::SSE::Event(event)) = event_result {
            if let Ok(json_data) = serde_json::from_str::<Value>(&event.data) {
                if let Some(parsed_array) = json_data["parsed"].as_array() {
                    if let Some(first_entry) = parsed_array.first() {
                        if let Some(raw_price_str) = first_entry["price"]["price"].as_str() {
                            if let Some(expo) = first_entry["price"]["expo"].as_i64() {
                                if let Ok(raw_price) = raw_price_str.parse::<f64>() {
                                    let adjusted_price = raw_price * 10f64.powi(expo as i32);
                                    update_high_low(adjusted_price);
                                }
                            }
                        }
                    }
                } 
            }
        }
    }
    Ok(())
}

