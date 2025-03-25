use plotters::prelude::*;
use std::error::Error;
use std::path::Path;

// Technical indicators frequently used in trading
pub mod indicators {
    use crate::models::Candle;
    
    // Exponential Moving Average (EMA)
    pub fn calculate_ema(candles: &[Candle], period: usize, index: usize) -> Option<f64> {
        if index < period - 1 || candles.len() <= index {
            return None;
        }
        
        if index == period - 1 {
            // First EMA is the SMA
            let sum: f64 = candles[0..period]
                .iter()
                .map(|candle| candle.close)
                .sum();
            return Some(sum / period as f64);
        }
        
        // EMA = Price(t) * k + EMA(y) * (1 - k)
        // where k = 2 / (period + 1)
        let k = 2.0 / (period as f64 + 1.0);
        let prev_ema = calculate_ema(candles, period, index - 1).unwrap();
        let ema = candles[index].close * k + prev_ema * (1.0 - k);
        
        Some(ema)
    }
    
    // Relative Strength Index (RSI)
    pub fn calculate_rsi(candles: &[Candle], period: usize, index: usize) -> Option<f64> {
        if index < period || candles.len() <= index {
            return None;
        }
        
        let mut gains = 0.0;
        let mut losses = 0.0;
        
        // Calculate average gains and losses
        for i in (index - period + 1)..=index {
            let change = candles[i].close - candles[i - 1].close;
            if change >= 0.0 {
                gains += change;
            } else {
                losses -= change; // Convert to positive
            }
        }
        
        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;
        
        // Calculate RSI
        if avg_loss == 0.0 {
            return Some(100.0);
        }
        
        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));
        
        Some(rsi)
    }
    
    // Bollinger Bands
    pub fn calculate_bollinger_bands(candles: &[Candle], period: usize, num_std_dev: f64, index: usize) -> Option<(f64, f64, f64)> {
        if index < period - 1 || candles.len() <= index {
            return None;
        }
        
        // Calculate Simple Moving Average
        let sum: f64 = candles[index - period + 1..=index]
            .iter()
            .map(|candle| candle.close)
            .sum();
        let sma = sum / period as f64;
        
        // Calculate Standard Deviation
        let variance: f64 = candles[index - period + 1..=index]
            .iter()
            .map(|candle| (candle.close - sma).powi(2))
            .sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();
        
        // Calculate Bollinger Bands
        let upper_band = sma + (std_dev * num_std_dev);
        let lower_band = sma - (std_dev * num_std_dev);
        
        Some((sma, upper_band, lower_band))
    }
    
    // Moving Average Convergence Divergence (MACD)
    pub fn calculate_macd(candles: &[Candle], fast_period: usize, slow_period: usize, signal_period: usize, index: usize) -> Option<(f64, f64, f64)> {
        if index < slow_period + signal_period - 2 || candles.len() <= index {
            return None;
        }
        
        // Calculate MACD line
        let fast_ema = calculate_ema(candles, fast_period, index)?;
        let slow_ema = calculate_ema(candles, slow_period, index)?;
        let macd_line = fast_ema - slow_ema;
        
        // Calculate signal line (EMA of MACD line)
        // For this simplified implementation, we'll manually calculate the signal line
        let mut macd_values = Vec::with_capacity(signal_period);
        for i in (index - signal_period + 1)..=index {
            if let (Some(fast), Some(slow)) = (calculate_ema(candles, fast_period, i), calculate_ema(candles, slow_period, i)) {
                macd_values.push(fast - slow);
            }
        }
        
        // Calculate EMA of MACD values for signal line
        let signal_line = macd_values.iter().sum::<f64>() / signal_period as f64;
        
        // MACD histogram
        let histogram = macd_line - signal_line;
        
        Some((macd_line, signal_line, histogram))
    }
    
    // Average True Range (ATR)
    pub fn calculate_atr(candles: &[Candle], period: usize, index: usize) -> Option<f64> {
        if index < period || candles.len() <= index {
            return None;
        }
        
        let mut tr_sum = 0.0;
        
        // Calculate True Range for last 'period' candles
        for i in (index - period + 1)..=index {
            let high = candles[i].high;
            let low = candles[i].low;
            let prev_close = candles[i - 1].close;
            
            // True Range is the greatest of the following:
            // 1. Current High - Current Low
            // 2. |Current High - Previous Close|
            // 3. |Current Low - Previous Close|
            let tr1 = high - low;
            let tr2 = (high - prev_close).abs();
            let tr3 = (low - prev_close).abs();
            
            let true_range = tr1.max(tr2).max(tr3);
            tr_sum += true_range;
        }
        
        // ATR is the average of True Range values
        Some(tr_sum / period as f64)
    }
}

// Risk management functions
pub mod risk {
    use crate::models::TradeDirection;
    
    // Calculate position size based on risk percentage
    pub fn position_size(account_balance: f64, risk_percent: f64, entry_price: f64, stop_loss: f64) -> f64 {
        let risk_amount = account_balance * (risk_percent / 100.0);
        let price_difference = (entry_price - stop_loss).abs();
        
        if price_difference == 0.0 {
            return 0.0;
        }
        
        risk_amount / price_difference
    }
    
    // Calculate stop loss price based on ATR
    pub fn atr_stop_loss(entry_price: f64, atr: f64, multiplier: f64, direction: TradeDirection) -> f64 {
        match direction {
            TradeDirection::Long => entry_price - (atr * multiplier),
            TradeDirection::Short => entry_price + (atr * multiplier),
        }
    }
    
    // Calculate take profit based on risk:reward ratio
    pub fn take_profit(entry_price: f64, stop_loss: f64, risk_reward_ratio: f64, direction: TradeDirection) -> f64 {
        let risk = (entry_price - stop_loss).abs();
        let reward = risk * risk_reward_ratio;
        
        match direction {
            TradeDirection::Long => entry_price + reward,
            TradeDirection::Short => entry_price - reward,
        }
    }
}

// Visualization utilities for backtesting results
pub fn plot_equity_curve<P: AsRef<Path>>(
    equity_curve: &[(chrono::DateTime<chrono::Utc>, f64)],
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(output_path.as_ref(), (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let min_date = equity_curve.first().map(|p| p.0).unwrap_or_else(|| chrono::Utc::now());
    let max_date = equity_curve.last().map(|p| p.0).unwrap_or_else(|| chrono::Utc::now());
    
    let min_equity = equity_curve
        .iter()
        .map(|p| p.1)
        .fold(f64::INFINITY, |a, b| a.min(b));
    let max_equity = equity_curve
        .iter()
        .map(|p| p.1)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Equity Curve", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(min_date..max_date, min_equity..max_equity)?;
    
    chart.configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .y_desc("Equity")
        .draw()?;
    
    chart.draw_series(LineSeries::new(
        equity_curve.iter().map(|p| (p.0, p.1)),
        &BLUE,
    ))?;
    
    Ok(())
}