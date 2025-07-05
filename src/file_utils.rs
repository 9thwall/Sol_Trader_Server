use chrono::{Duration, Utc};
use serde_json::{json, Value, Map};
use std::fs;
use std::path::Path;
use crate::config::FILE_PATH;

pub fn update_high_low(current_price: f64) {
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let today = now.format("%Y-%m-%d").to_string();
    let cutoff_date = (now - Duration::days(5)).format("%Y-%m-%d").to_string();

    let mut changed = false;
    let mut high = current_price;
    let mut low = current_price;
    let mut high_time = now_str.clone();
    let mut low_time = now_str.clone();


    println!("💰 Current price: {}", current_price);
    println!("📅 Today: {}", today);

    // Load existing JSON if the file exists
    let data = if Path::new(FILE_PATH.as_str()).exists() {
        fs::read_to_string(FILE_PATH.as_str())
            .ok()
            .and_then(|content| serde_json::from_str::<Value>(&content).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };

    // Retain only last 5 days
    let keys_to_keep: Vec<String> = data.as_object()
        .map(|obj| {
            obj.keys()
                .filter(|k| *k >= &cutoff_date)
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    let mut pruned: Map<String, Value> = Map::new();
    for key in keys_to_keep {
        if let Some(entry) = data.get(&key) {
            pruned.insert(key, entry.clone());
        }
    }

    // Step 1: Check if today's entry exists
    let today_entry = pruned.get(&today);

    if today_entry.is_none() {
        println!("📅 No entry for today. First tick gets inserted.");
        changed = true;
    } else if let Some(Value::Object(obj)) = today_entry {
        if let Some(prev_high) = obj.get("high").and_then(|v| v.as_f64()) {
            if current_price > prev_high {
                high = current_price;
                high_time = now_str.clone();
                changed = true;
            } else {
                high = prev_high;
                high_time = obj.get("high_timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&now_str)
                    .to_string();
            }
        }

        if let Some(prev_low) = obj.get("low").and_then(|v| v.as_f64()) {
            if current_price < prev_low {
                low = current_price;
                low_time = now_str.clone();
                changed = true;
            } else {
                low = prev_low;
                low_time = obj.get("low_timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&now_str)
                    .to_string();
            }
        }
    }

    println!("🔍 Did data change? {}", changed);

    if changed {
        pruned.insert(today.clone(), json!({
            "high": high,
            "high_timestamp": high_time,
            "low": low,
            "low_timestamp": low_time,
            "last_seen_price": current_price,
            "last_updated": now_str
        }));

        match serde_json::to_string_pretty(&Value::Object(pruned)) {
            Ok(serialized) => {
                if let Err(err) = fs::write(FILE_PATH.as_str(), serialized) {
                    eprintln!("⚠️ Failed to write file to {}: {}", FILE_PATH.as_str(), err);
                } else {
                    println!(
                        "📈 Updating JSON: high = {}, low = {}, current = {}",
                        high, low, current_price
                    );
                }
            }
            Err(e) => eprintln!("⚠️ Failed to serialize JSON: {}", e),
        }
    }
}