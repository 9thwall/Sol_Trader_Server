mod pyth_tracker;
mod file_utils;
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_path = std::env::var("ENV_PATH").unwrap_or_else(|_| ".env".to_string());
    if let Err(e) = dotenv::from_path(&env_path) {
        eprintln!("⚠️ Failed to load .env file from '{}': {e}", env_path);
    } else {
        println!("✅ Loaded .env file from: {}", env_path);
    }
    //dotenv::from_path("/etc/solana-tracker/.env").ok();
    pyth_tracker::run_pyth_tracker().await?;
    Ok(())
}