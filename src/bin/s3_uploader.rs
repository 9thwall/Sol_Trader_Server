use std::collections::BTreeMap;
use std::{env, fs};
use std::path::Path;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_types::region::Region;
use chrono::{Duration, Utc};
use once_cell::sync::Lazy;
use serde_json::{Map, Value};
use solana_tracker::config::FILE_PATH;


pub static BUCKET_NAME: Lazy<String> = Lazy::new(|| {
    env::var("BUCKET_NAME").expect("Missing BUCKET_NAME env var")
});

pub static S3_STORE_MAX_DAYS: Lazy<i64> = Lazy::new(|| {
    env::var("S3_STORE_MAX_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(90)
});

pub static AWS_REGION: Lazy<String> = Lazy::new(|| {
    env::var("AWS_REGION").unwrap_or_else(|_| "us-west-1".to_string())
});


#[tokio::main]
async fn main() {
    dotenv::from_path("/etc/solana-tracker/.env").ok();

    // if let Err(e) = s3_uploader::update_s3_history().await {
    //     eprintln!("Upload failed: {e}");
    // }

    match update_s3_history().await {
        Ok(_) => println!("✅ S3 history successfully uploaded."),
        Err(e) => eprintln!("❌ S3 upload failed: {e}"),
    }
}


//This runs right after midnight.
//So we need to get yesterday's date data.
async fn update_s3_history() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Read the local 5-day file
    if !Path::new(FILE_PATH.as_str()).exists() {
        eprintln!("⚠️ Local high/low file not found.");
        return Ok(());
    }
    let local_content = fs::read_to_string(FILE_PATH.as_str())?;
    let local_json: Value = serde_json::from_str(&local_content)?;

    let yesterday = (Utc::now() - Duration::days(1)).format("%Y-%m-%d").to_string();

    //Getting yesterday data because this will run after midnight!
    let Some(yesterday_data) = local_json.get(&yesterday) else {
        eprintln!("⚠️ No entry found for yesterday in local file.");
        return Ok(());
    };

    // Step 2: Download the current S3 history file
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new(AWS_REGION.as_str()));
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = Client::new(&config);

    let mut is_new_history = false;

    let s3_key = "history.json".to_string();
    let existing_history = match client
        .get_object()
        .bucket(BUCKET_NAME.as_str())
        .key(&s3_key)
        .send()
        .await
    {
        Ok(resp) => {
            let bytes = resp.body.collect().await?.into_bytes();
            let text = String::from_utf8_lossy(&bytes);
            serde_json::from_str::<Map<String, Value>>(&text).unwrap_or_default()
        }
        Err(_) => {
            eprintln!("ℹ️ No existing history found. Initializing new one.");
            is_new_history = true;
            Map::new()
        }
    };

    // Step 3: Merge today's data into a BTreeMap for sorted keys
    let mut history: BTreeMap<String, Value> = existing_history.into_iter().collect();
    history.insert(yesterday, yesterday_data.clone());

    // Step 4: Prune to max 90 days
    let cutoff = (Utc::now() - Duration::days(*S3_STORE_MAX_DAYS)).format("%Y-%m-%d").to_string();
    history.retain(|date, _| date >= &cutoff);
    println!("ℹ️ Storing {} days in S3 history.", history.len());

    // Step 5: Upload the new version to S3
    let payload = serde_json::to_string_pretty(&history)?;

    let upload_result = client.put_object()
        .bucket(BUCKET_NAME.as_str())
        .key(&s3_key)
        .body(ByteStream::from(payload.into_bytes()))
        .content_type("application/json")
        .send()
        .await;

    match upload_result {
        Ok(_) => {
            if is_new_history {
                println!("✅ Created new history.json file in S3.");
            } else {
                println!("✅ Updated history.json in S3.");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to upload to S3");
            eprintln!("   ├─ Bucket: {}", BUCKET_NAME.as_str());
            eprintln!("   ├─ Key: {}", s3_key);
            eprintln!("   └─ Error: {:?}", e);
        }
    }
    Ok(())
}