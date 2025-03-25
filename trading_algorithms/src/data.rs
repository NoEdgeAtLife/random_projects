use crate::models::{Candle, MarketData};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub async fn fetch_historical_data(
    symbol: &str,
    start_date: &str,
    end_date: &str,
) -> Result<MarketData, Box<dyn Error>> {
    // In a real application, this would use an API to fetch real market data
    // For now, we'll simulate loading data
    
    println!("Fetching historical data for {} from {} to {}", symbol, start_date, end_date);
    
    // For demo purposes, we'll generate some dummy data
    let candles = generate_dummy_data(symbol, start_date, end_date)?;
    
    Ok(MarketData {
        symbol: symbol.to_string(),
        timeframe: "1D".to_string(),
        candles,
    })
}

pub fn load_csv_data(csv_path: &Path, symbol: &str) -> Result<MarketData, Box<dyn Error>> {
    let file = File::open(csv_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(file);
    
    let mut candles = Vec::new();
    
    for result in reader.records() {
        let record = result?;
        if record.len() < 6 {
            continue;
        }
        
        // Parse CSV fields (assuming format: timestamp,open,high,low,close,volume)
        let timestamp = record[0].parse::<i64>()?;
        let timestamp = Utc.timestamp_opt(timestamp, 0).single()
            .ok_or_else(|| "Invalid timestamp")?;
            
        let open = record[1].parse::<f64>()?;
        let high = record[2].parse::<f64>()?;
        let low = record[3].parse::<f64>()?;
        let close = record[4].parse::<f64>()?;
        let volume = record[5].parse::<f64>()?;
        
        candles.push(Candle {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        });
    }
    
    Ok(MarketData {
        symbol: symbol.to_string(),
        timeframe: "1D".to_string(),
        candles,
    })
}

fn generate_dummy_data(
    _symbol: &str,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<Candle>, Box<dyn Error>> {
    let start = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", start_date), "%Y-%m-%d %H:%M:%S")?;
    let end = NaiveDateTime::parse_from_str(&format!("{} 23:59:59", end_date), "%Y-%m-%d %H:%M:%S")?;
    
    let start_timestamp = DateTime::<Utc>::from_naive_utc_and_offset(start, Utc);
    let end_timestamp = DateTime::<Utc>::from_naive_utc_and_offset(end, Utc);
    
    let mut candles = Vec::new();
    let mut current_price = 100.0;
    let mut current_time = start_timestamp;
    
    let day_seconds = 24 * 60 * 60;
    
    while current_time <= end_timestamp {
        // Generate some random price movement
        let change_percent = (rand() * 2.0 - 1.0) * 0.02; // -2% to +2%
        let open = current_price;
        let close = open * (1.0 + change_percent);
        let high = open.max(close) * (1.0 + rand() * 0.01);
        let low = open.min(close) * (1.0 - rand() * 0.01);
        let volume = 10000.0 + rand() * 90000.0;
        
        candles.push(Candle {
            timestamp: current_time,
            open,
            high,
            low,
            close,
            volume,
        });
        
        current_price = close;
        current_time = current_time + chrono::Duration::seconds(day_seconds);
    }
    
    Ok(candles)
}

// Simple deterministic random number generator for demo purposes
fn rand() -> f64 {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as f64;
    
    (now / 1_000_000_000.0).fract()
}