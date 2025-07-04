# Solana Tracker

A Rust-based tool that tracks Solana (SOL) high/low prices, stores the last 5 days locally, and pushes 90-day historical data to AWS S3 nightly.

## 🧰 Features

- Live price tracking via Pyth
- Local JSON logging for 5 days
- AWS S3 uploads for historical record (90 days)
- Production-ready for systemd

## 📦 Requirements

- Rust (latest stable)
- AWS credentials via `.env`
- S3 bucket already created
- A Linux server (e.g., CentOS) for deployment

## 🔧 Setup

```bash
# Clone the repo
git clone https://github.com/yourusername/solana-tracker.git
cd solana-tracker

# Create your .env file
touch .env