use once_cell::sync::Lazy;
use std::env;

pub static S3_STORE_MAX_DAYS: Lazy<i64> = Lazy::new(|| {
    env::var("S3_STORE_MAX_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(90)
});

// Load env vars at runtime
pub static AWS_REGION: Lazy<String> = Lazy::new(|| {
    env::var("AWS_REGION").unwrap_or_else(|_| "us-west-1".to_string())
});

pub static SSE_URL: Lazy<String> = Lazy::new(|| {
    env::var("SSE_URL").expect("Missing SSE_URL env var")
});

pub static FILE_PATH: Lazy<String> = Lazy::new(|| {
    env::var("FILE_PATH").unwrap_or_else(|_| "/var/lib/solana-tracker/pyth_highlow.json".to_string())
});

pub static BUCKET_NAME: Lazy<String> = Lazy::new(|| {
    env::var("BUCKET_NAME").expect("Missing BUCKET_NAME env var")
});
