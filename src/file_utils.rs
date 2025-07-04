use std::collections::HashMap;
use chrono::{Duration, Utc};
use serde_json::{json, Value};
use std::fs;
use crate::config::FILE_PATH;

pub fn update_high_low(current_price: f64) {
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let today = now.format("%Y-%m-%d").to_string();
    let cutoff_date = (now - Duration::days(5)).format("%Y-%m-%d");

    let mut changed = false;
    let mut high = current_price;
    let mut low = current_price;
    let mut high_time = now_str.as_str();
    let mut low_time = now_str.as_str();

    // Load existing JSON
    let mut pruned: HashMap<String, Value> = HashMap::with_capacity(6);

    if let Ok(content) = fs::read_to_string(FILE_PATH.as_str()) {
        if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&content) {
            let cutoff_str = cutoff_date.to_string();
            for (key, val) in map {
                if key >= cutoff_str {
                    pruned.insert(key, val);
                }
            }
        }
    }

    // Use today’s record if exists
    if let Some(Value::Object(today_data)) = pruned.get(&today) {
        if let Some(prev_high) = today_data.get("high").and_then(|v| v.as_f64()) {
            if current_price > prev_high {
                high = current_price;
                high_time = &now_str;
                changed = true;
            } else {
                high = prev_high;
                high_time = today_data.get("high_timestamp").and_then(|v| v.as_str()).unwrap_or(&now_str);
            }
        }

        if let Some(prev_low) = today_data.get("low").and_then(|v| v.as_f64()) {
            if current_price < prev_low {
                low = current_price;
                low_time = &now_str;
                changed = true;
            } else {
                low = prev_low;
                low_time = today_data.get("low_timestamp").and_then(|v| v.as_str()).unwrap_or(&now_str);
            }
        }
    } else {
        changed = true; // new day
    }

    if changed {
        pruned.insert(today, json!({
            "high": high,
            "high_timestamp": high_time,
            "low": low,
            "low_timestamp": low_time,
            "last_seen_price": current_price,
            "last_updated": now_str
        }));

        match serde_json::to_string_pretty(&Value::Object(pruned.into_iter().collect())) {
            Ok(serialized) => {
                if let Err(err) = fs::write(FILE_PATH.as_str(), serialized) {

                    println!("{}", FILE_PATH.as_str());

                    eprintln!("⚠️ Failed to write file: {}", err);
                }
            }
            Err(e) => eprintln!("⚠️ Failed to serialize JSON: {}", e),
        }
    }
}