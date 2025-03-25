use crate::models::{Candle, MarketData, Trade, TradeDirection};
use chrono::{DateTime, Utc};
use std::error::Error;

/// Execution Algorithm trait for implementing various order execution strategies
pub trait ExecutionAlgorithm {
    fn name(&self) -> &str;
    fn execute(&self, data: &MarketData, order_size: f64, direction: TradeDirection, start_time: DateTime<Utc>, end_time: Option<DateTime<Utc>>) -> Result<Vec<Trade>, Box<dyn Error>>;
}

/// Volume-Weighted Average Price (VWAP) execution algorithm
/// 
/// VWAP tries to execute orders close to the volume-weighted average price
/// by distributing the order in proportion to expected volume profile.
pub struct VWAP {
    pub name: String,
    pub num_buckets: usize,
    pub participation_rate: f64, // Target participation rate (0.0-1.0)
}

impl VWAP {
    pub fn new(num_buckets: usize, participation_rate: f64) -> Self {
        Self {
            name: format!("VWAP_{}_buckets_{:.2}rate", num_buckets, participation_rate),
            num_buckets,
            participation_rate: participation_rate.clamp(0.0, 1.0),
        }
    }

    // Calculate historical volume profile from past data
    #[allow(dead_code)]
    fn calculate_volume_profile(&self, historical_data: &[MarketData]) -> Vec<f64> {
        // Initialize volume buckets
        let mut volume_profile = vec![0.0; self.num_buckets];
        let mut total_volume = 0.0;
        
        // Aggregate volumes across historical days
        for data in historical_data {
            let candles_per_bucket = (data.candles.len() as f64 / self.num_buckets as f64).ceil() as usize;
            
            for (i, candle) in data.candles.iter().enumerate() {
                let bucket_idx = i / candles_per_bucket;
                if bucket_idx < self.num_buckets {
                    volume_profile[bucket_idx] += candle.volume;
                    total_volume += candle.volume;
                }
            }
        }
        
        // Normalize volume profile
        if total_volume > 0.0 {
            for vol in &mut volume_profile {
                *vol /= total_volume;
            }
        }
        
        volume_profile
    }
}

impl ExecutionAlgorithm for VWAP {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, data: &MarketData, order_size: f64, direction: TradeDirection, start_time: DateTime<Utc>, end_time: Option<DateTime<Utc>>) -> Result<Vec<Trade>, Box<dyn Error>> {
        let mut trades = Vec::new();
        let candles = &data.candles;
        
        if candles.is_empty() {
            return Ok(trades);
        }

        // Filter candles within the trading window
        let end_time = end_time.unwrap_or_else(|| candles.last().unwrap().timestamp);
        let trading_candles: Vec<&Candle> = candles
            .iter()
            .filter(|c| c.timestamp >= start_time && c.timestamp <= end_time)
            .collect();
            
        if trading_candles.is_empty() {
            return Ok(trades);
        }

        // Use actual volume profile from data if available, otherwise use equal distribution
        let buckets = self.num_buckets.min(trading_candles.len());
        let candles_per_bucket = (trading_candles.len() as f64 / buckets as f64).ceil() as usize;
        
        let mut total_volume = trading_candles.iter().map(|c| c.volume).sum::<f64>();
        let mut remaining_size = order_size;
        
        // Generate trades based on volume profile
        for (i, candle_chunk) in trading_candles.chunks(candles_per_bucket).enumerate() {
            let bucket_volume = candle_chunk.iter().map(|c| c.volume).sum::<f64>();
            let volume_ratio = bucket_volume / total_volume;
            
            // Calculate size to execute in this bucket
            let bucket_size = order_size * volume_ratio * self.participation_rate;
            let size_to_execute = remaining_size.min(bucket_size);
            
            if size_to_execute > 0.0 {
                // Distribute size within the bucket proportionally to each candle's volume
                for candle in candle_chunk {
                    let candle_volume_ratio = candle.volume / bucket_volume;
                    let candle_size = size_to_execute * candle_volume_ratio;
                    
                    if candle_size > 0.0 {
                        let costs = candle.close * candle_size * 0.001; // 0.1% commission
                        
                        trades.push(Trade {
                            timestamp: candle.timestamp,
                            symbol: data.symbol.clone(),
                            direction,
                            price: candle.close,
                            size: candle_size,
                            costs,
                        });
                        
                        remaining_size -= candle_size;
                    }
                }
            }
            
            // Adjust total volume for next buckets
            total_volume -= bucket_volume;
        }
        
        // If there's any remaining size due to rounding, execute at the last candle
        if remaining_size > 0.01 {
            let last_candle = trading_candles.last().unwrap();
            let costs = last_candle.close * remaining_size * 0.001;
            
            trades.push(Trade {
                timestamp: last_candle.timestamp,
                symbol: data.symbol.clone(),
                direction,
                price: last_candle.close,
                size: remaining_size,
                costs,
            });
        }

        Ok(trades)
    }
}

/// Time-Weighted Average Price (TWAP) execution algorithm
/// 
/// TWAP evenly distributes the order over time in fixed-size chunks.
pub struct TWAP {
    pub name: String,
    pub num_slices: usize,
}

impl TWAP {
    pub fn new(num_slices: usize) -> Self {
        Self {
            name: format!("TWAP_{}_slices", num_slices),
            num_slices,
        }
    }
}

impl ExecutionAlgorithm for TWAP {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, data: &MarketData, order_size: f64, direction: TradeDirection, start_time: DateTime<Utc>, end_time: Option<DateTime<Utc>>) -> Result<Vec<Trade>, Box<dyn Error>> {
        let mut trades = Vec::new();
        let candles = &data.candles;
        
        if candles.is_empty() {
            return Ok(trades);
        }

        // Filter candles within the trading window
        let end_time = end_time.unwrap_or_else(|| candles.last().unwrap().timestamp);
        let trading_candles: Vec<&Candle> = candles
            .iter()
            .filter(|c| c.timestamp >= start_time && c.timestamp <= end_time)
            .collect();
            
        if trading_candles.is_empty() {
            return Ok(trades);
        }

        // Calculate slices (use min of requested slices or available candles)
        let num_slices = self.num_slices.min(trading_candles.len());
        let slice_size = order_size / num_slices as f64;
        let candles_per_slice = (trading_candles.len() as f64 / num_slices as f64).ceil() as usize;
        
        // Generate trades based on time slices
        let mut remaining_size = order_size;
        
        for (i, candles_chunk) in trading_candles.chunks(candles_per_slice).enumerate() {
            if candles_chunk.is_empty() || remaining_size <= 0.0 {
                break;
            }
            
            // Use the middle candle of each chunk for execution
            let candle_idx = candles_chunk.len() / 2;
            let candle = candles_chunk[candle_idx];
            
            let slice_execution_size = slice_size.min(remaining_size);
            let costs = candle.close * slice_execution_size * 0.001; // 0.1% commission
            
            trades.push(Trade {
                timestamp: candle.timestamp,
                symbol: data.symbol.clone(),
                direction,
                price: candle.close,
                size: slice_execution_size,
                costs,
            });
            
            remaining_size -= slice_execution_size;
        }
        
        // If there's any remaining size due to rounding, execute at the last candle
        if remaining_size > 0.01 {
            let last_candle = trading_candles.last().unwrap();
            let costs = last_candle.close * remaining_size * 0.001;
            
            trades.push(Trade {
                timestamp: last_candle.timestamp,
                symbol: data.symbol.clone(),
                direction,
                price: last_candle.close,
                size: remaining_size,
                costs,
            });
        }

        Ok(trades)
    }
}

/// Implementation Shortfall (IS) execution algorithm
/// 
/// Implementation Shortfall aims to minimize the difference between the decision price
/// and the average execution price, balancing market impact and timing risk.
pub struct ImplementationShortfall {
    pub name: String,
    pub urgency: f64, // 0.0 (passive) to 1.0 (urgent)
    pub initial_pct: f64, // Initial execution percentage
    pub risk_aversion: f64,
}

impl ImplementationShortfall {
    pub fn new(urgency: f64, initial_pct: f64, risk_aversion: f64) -> Self {
        let urgency = urgency.clamp(0.0, 1.0);
        let initial_pct = initial_pct.clamp(0.0, 1.0);
        
        Self {
            name: format!("IS_urgency{:.2}", urgency),
            urgency,
            initial_pct,
            risk_aversion,
        }
    }
    
    // Calculate market impact cost based on order size and liquidity
    fn estimate_market_impact(&self, price: f64, size: f64, avg_volume: f64) -> f64 {
        // Simple market impact model: impact = price * size^0.6 / (avg_volume * 10)
        let market_impact = price * (size.powf(0.6) / (avg_volume * 10.0));
        market_impact.min(price * 0.01) // Cap impact at 1% of price
    }
    
    // Calculate the optimal trading schedule based on Almgren-Chriss model
    fn calculate_trading_schedule(&self, 
        order_size: f64, 
        num_periods: usize,
        volatility: f64, 
        avg_volume: f64,
        avg_price: f64
    ) -> Vec<f64> {
        let mut schedule = Vec::with_capacity(num_periods);
        
        // Simplified Almgren-Chriss model parameters
        let market_impact_factor: f64 = 0.1;
        let temp_impact_factor: f64 = 0.05;
        let tau = self.risk_aversion * volatility.powi(2);
        
        // Calculate κ (kappa) parameter
        let kappa = (market_impact_factor / (temp_impact_factor * 0.5)).sqrt();
        
        // Calculate remaining size at each period
        let mut remaining = order_size;
        
        // Initial trade based on urgency
        let initial_trade = order_size * self.initial_pct * self.urgency;
        schedule.push(initial_trade);
        remaining -= initial_trade;
        
        // Calculate exponential decay for remaining size
        let decay_factor = (-kappa * tau).exp();
        
        for i in 1..num_periods {
            let is_last_period = i == num_periods - 1;
            
            if is_last_period {
                // Execute all remaining size in last period
                schedule.push(remaining);
            } else {
                // Execute based on exponential decay
                let size_to_execute = if self.urgency > 0.8 {
                    // High urgency: more aggressive execution
                    remaining / (num_periods - i) as f64
                } else {
                    // Normal urgency: exponential decay
                    remaining * (1.0 - decay_factor)
                };
                
                schedule.push(size_to_execute);
                remaining -= size_to_execute;
            }
        }
        
        schedule
    }
}

impl ExecutionAlgorithm for ImplementationShortfall {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, data: &MarketData, order_size: f64, direction: TradeDirection, start_time: DateTime<Utc>, end_time: Option<DateTime<Utc>>) -> Result<Vec<Trade>, Box<dyn Error>> {
        let mut trades = Vec::new();
        let candles = &data.candles;
        
        if candles.is_empty() {
            return Ok(trades);
        }

        // Filter candles within the trading window
        let end_time = end_time.unwrap_or_else(|| candles.last().unwrap().timestamp);
        let trading_candles: Vec<&Candle> = candles
            .iter()
            .filter(|c| c.timestamp >= start_time && c.timestamp <= end_time)
            .collect();
            
        if trading_candles.is_empty() {
            return Ok(trades);
        }
        
        // Calculate average volume and volatility for market impact estimation
        let avg_volume: f64 = trading_candles.iter().map(|c| c.volume).sum::<f64>() / trading_candles.len() as f64;
        let avg_price: f64 = trading_candles.iter().map(|c| c.close).sum::<f64>() / trading_candles.len() as f64;
        
        // Calculate price volatility
        let returns: Vec<f64> = trading_candles.windows(2)
            .map(|w| (w[1].close - w[0].close) / w[0].close)
            .collect();
            
        let volatility = if returns.is_empty() {
            0.01 // Default volatility if we can't calculate it
        } else {
            let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns.iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>() / returns.len() as f64;
            variance.sqrt()
        };
        
        // Determine the number of periods for execution
        let num_periods = (trading_candles.len() as f64 * 0.8).round() as usize;
        let num_periods = num_periods.max(2); // Ensure at least 2 periods
        
        // Calculate trading schedule
        let schedule = self.calculate_trading_schedule(
            order_size, 
            num_periods,
            volatility,
            avg_volume,
            avg_price
        );
        
        // Execute trades according to schedule
        let candles_per_period = trading_candles.len() / num_periods;
        let candles_per_period = candles_per_period.max(1);
        
        for (i, size_to_execute) in schedule.iter().enumerate() {
            if *size_to_execute <= 0.0 {
                continue;
            }
            
            // Select candle for this period
            let period_start = i * candles_per_period;
            let candle_idx = if self.urgency > 0.7 {
                // For high urgency, execute at beginning of period
                period_start
            } else {
                // For low urgency, execute at a random point within period
                period_start + (i % candles_per_period).min(trading_candles.len() - period_start - 1)
            };
            
            // Ensure we don't go out of bounds
            let candle_idx = candle_idx.min(trading_candles.len() - 1);
            let candle = trading_candles[candle_idx];
            
            // Calculate market impact
            let base_price = candle.close;
            let impact = self.estimate_market_impact(base_price, *size_to_execute, avg_volume);
            
            // Adjust price based on market impact and direction
            let execution_price = match direction {
                TradeDirection::Long => base_price + impact, // Buy price is higher due to impact
                TradeDirection::Short => base_price - impact, // Sell price is lower due to impact
            };
            
            let costs = execution_price * *size_to_execute * 0.001; // 0.1% commission
            
            trades.push(Trade {
                timestamp: candle.timestamp,
                symbol: data.symbol.clone(),
                direction,
                price: execution_price,
                size: *size_to_execute,
                costs,
            });
        }

        Ok(trades)
    }
}

/// Adaptive Market Execution algorithm
/// 
/// Dynamically adjusts execution based on real-time market conditions
pub struct AdaptiveMarketExecution {
    pub name: String,
    pub base_participation_rate: f64,
    pub min_participation_rate: f64,
    pub max_participation_rate: f64,
    pub volatility_factor: f64,
    pub momentum_lookback: usize,
}

impl AdaptiveMarketExecution {
    pub fn new(
        base_rate: f64, 
        min_rate: f64, 
        max_rate: f64,
        volatility_factor: f64,
        momentum_lookback: usize
    ) -> Self {
        Self {
            name: format!("AdaptiveExecution_{:.2}base", base_rate),
            base_participation_rate: base_rate.clamp(0.0, 1.0),
            min_participation_rate: min_rate.clamp(0.0, 1.0),
            max_participation_rate: max_rate.clamp(0.0, 1.0),
            volatility_factor,
            momentum_lookback: momentum_lookback.max(5),
        }
    }
    
    // Calculate price momentum 
    fn calculate_momentum(&self, candles: &[&Candle], current_idx: usize) -> f64 {
        let lookback = self.momentum_lookback.min(current_idx);
        
        if lookback == 0 {
            return 0.0;
        }
        
        let current_price = candles[current_idx].close;
        let past_price = candles[current_idx - lookback].close;
        
        (current_price / past_price - 1.0) * 100.0 // Percentage change
    }
    
    // Calculate local volatility
    fn calculate_volatility(&self, candles: &[&Candle], current_idx: usize) -> f64 {
        let lookback = self.momentum_lookback.min(current_idx);
        
        if lookback < 2 {
            return 0.01; // Default volatility
        }
        
        let prices: Vec<f64> = candles[current_idx - lookback..=current_idx]
            .iter()
            .map(|c| c.close)
            .collect();
            
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
            
        variance.sqrt() / mean // Coefficient of variation
    }
    
    // Adjust participation rate based on market conditions
    fn adjust_participation_rate(&self, 
        base_rate: f64, 
        momentum: f64, 
        volatility: f64,
        direction: TradeDirection
    ) -> f64 {
        // Base adjustment from volatility - higher volatility generally means more careful execution
        let volatility_adjustment = -volatility * self.volatility_factor;
        
        // Momentum adjustment depends on direction
        let momentum_adjustment = match direction {
            // For buys, positive momentum means prices moving against us, so be more aggressive
            TradeDirection::Long => if momentum > 0.0 { momentum * 0.01 } else { momentum * 0.005 },
            
            // For sells, negative momentum means prices moving against us, so be more aggressive
            TradeDirection::Short => if momentum < 0.0 { -momentum * 0.01 } else { -momentum * 0.005 },
        };
        
        // Combine adjustments
        let adjusted_rate = base_rate + volatility_adjustment + momentum_adjustment;
        
        // Clamp to allowed range
        adjusted_rate.clamp(self.min_participation_rate, self.max_participation_rate)
    }
}

impl ExecutionAlgorithm for AdaptiveMarketExecution {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, data: &MarketData, order_size: f64, direction: TradeDirection, start_time: DateTime<Utc>, end_time: Option<DateTime<Utc>>) -> Result<Vec<Trade>, Box<dyn Error>> {
        let mut trades = Vec::new();
        let candles = &data.candles;
        
        if candles.is_empty() {
            return Ok(trades);
        }

        // Filter candles within the trading window
        let end_time = end_time.unwrap_or_else(|| candles.last().unwrap().timestamp);
        let trading_candles: Vec<&Candle> = candles
            .iter()
            .filter(|c| c.timestamp >= start_time && c.timestamp <= end_time)
            .collect();
            
        if trading_candles.is_empty() {
            return Ok(trades);
        }

        let mut remaining_size = order_size;
        let avg_price = trading_candles.iter().map(|c| c.close).sum::<f64>() / trading_candles.len() as f64;
        
        // Use a moving window for volatility and momentum calculations
        let min_window = self.momentum_lookback + 1;
        
        // Process each candle where we have enough history for our indicators
        for i in min_window..trading_candles.len() {
            if remaining_size <= 0.0 {
                break;
            }
            
            let candle = trading_candles[i];
            let momentum = self.calculate_momentum(&trading_candles, i);
            let volatility = self.calculate_volatility(&trading_candles, i);
            
            // Calculate participation rate for this candle
            let participation_rate = self.adjust_participation_rate(
                self.base_participation_rate,
                momentum,
                volatility,
                direction
            );
            
            // Estimate volume share for this candle
            let candle_volume_share = candle.volume / 
                trading_candles[i-min_window..=i].iter().map(|c| c.volume).sum::<f64>();
                
            // Calculate size to execute with this candle
            let size_to_execute = (remaining_size * candle_volume_share * participation_rate)
                .min(remaining_size);
                
            if size_to_execute > 0.01 { // Minimum execution size
                let costs = candle.close * size_to_execute * 0.001; // 0.1% commission
                
                trades.push(Trade {
                    timestamp: candle.timestamp,
                    symbol: data.symbol.clone(),
                    direction,
                    price: candle.close,
                    size: size_to_execute,
                    costs,
                });
                
                remaining_size -= size_to_execute;
            }
        }
        
        // If there's any remaining size, execute at the last candle
        if remaining_size > 0.01 {
            let last_candle = trading_candles.last().unwrap();
            let costs = last_candle.close * remaining_size * 0.001;
            
            trades.push(Trade {
                timestamp: last_candle.timestamp,
                symbol: data.symbol.clone(),
                direction,
                price: last_candle.close,
                size: remaining_size,
                costs,
            });
        }

        Ok(trades)
    }
}

/// Factory function to create execution algorithms by name
pub fn create_execution_algorithm(name: &str) -> Box<dyn ExecutionAlgorithm> {
    match name {
        "vwap" => Box::new(VWAP::new(10, 0.3)),
        "twap" => Box::new(TWAP::new(12)),
        "implementation_shortfall" | "is" => Box::new(ImplementationShortfall::new(0.5, 0.2, 0.3)),
        "adaptive" => Box::new(AdaptiveMarketExecution::new(0.3, 0.1, 0.6, 0.5, 10)),
        _ => Box::new(TWAP::new(10)), // Default
    }
}