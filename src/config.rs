use once_cell::sync::Lazy;
use std::env;

// Load shared env vars at runtime
pub static FILE_PATH: Lazy<String> = Lazy::new(|| {
    env::var("FILE_PATH").unwrap_or_else(|_| "/var/lib/solana-tracker/pyth_highlow.json".to_string())
});