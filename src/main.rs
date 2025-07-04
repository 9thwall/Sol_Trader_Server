mod pyth_tracker;
mod file_utils;
mod s3_uploader;
mod config;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::from_path(".env").ok();
    //dotenv::from_path("/etc/solana-tracker/.env").ok();
    
    //pyth_tracker::run_pyth_tracker().await?;

    // if let Err(e) = s3_uploader::update_s3_history().await {
    //     eprintln!("Upload failed: {e}");
    // }
    
    // Spawn nightly S3 history updater
    tokio::spawn(async {
        use tokio::time::{sleep_until, Duration, Instant};
        use chrono::{Utc};

        loop {
            let now = Utc::now();

            // Next midnight UTC
            let next_midnight = Utc::now()
                .date_naive()
                .succ_opt().unwrap()
                .and_hms_opt(0, 1, 0).unwrap() // set to 00:01 UTC
                .and_utc();

            let wait_duration = (next_midnight - now).to_std().unwrap_or(Duration::from_secs(86400));

            println!("⏳ Sleeping for {} seconds until next update...", wait_duration.as_secs());
            sleep_until(Instant::now() + wait_duration).await;

            println!("🌙 Running nightly S3 history update...");
            match s3_uploader::update_s3_history().await {
                Ok(_) => println!("✅ S3 history updated."),
                Err(e) => eprintln!("❌ S3 update failed: {e}"),
            }
        }
    });

    Ok(())
}