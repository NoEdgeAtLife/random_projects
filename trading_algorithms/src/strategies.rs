use crate::models::{Candle, MarketData, Trade, TradeDirection};
use std::error::Error;

pub trait Strategy {
    fn name(&self) -> &str;
    fn execute(&self, data: &MarketData) -> Result<Vec<Trade>, Box<dyn Error>>;
}

// Moving Average Crossover Strategy
pub struct MovingAverageCrossover {
    pub name: String,
    pub fast_period: usize,
    pub slow_period: usize,
}

impl MovingAverageCrossover {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            name: format!("MA_{}_{}_Crossover", fast_period, slow_period),
            fast_period,
            slow_period,
        }
    }

    fn calculate_ma(&self, candles: &[Candle], period: usize, index: usize) -> Option<f64> {
        if index < period - 1 || candles.len() <= index {
            return None;
        }

        let sum: f64 = candles[index - period + 1..=index]
            .iter()
            .map(|candle| candle.close)
            .sum();

        Some(sum / period as f64)
    }
}

impl Strategy for MovingAverageCrossover {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, data: &MarketData) -> Result<Vec<Trade>, Box<dyn Error>> {
        let mut trades = Vec::new();
        let candles = &data.candles;

        if candles.len() < self.slow_period {
            return Ok(trades); // Not enough data
        }

        let mut position: Option<TradeDirection> = None;

        for i in self.slow_period..candles.len() {
            let fast_ma = self.calculate_ma(candles, self.fast_period, i).unwrap();
            let slow_ma = self.calculate_ma(candles, self.slow_period, i).unwrap();
            let prev_fast_ma = self.calculate_ma(candles, self.fast_period, i - 1).unwrap();
            let prev_slow_ma = self.calculate_ma(candles, self.slow_period, i - 1).unwrap();

            // Detect crossing
            let cross_above = prev_fast_ma <= prev_slow_ma && fast_ma > slow_ma;
            let cross_below = prev_fast_ma >= prev_slow_ma && fast_ma < slow_ma;

            // Generate trades based on crossovers
            if cross_above && position != Some(TradeDirection::Long) {
                // Close short position if exists
                if position == Some(TradeDirection::Short) {
                    trades.push(Trade {
                        timestamp: candles[i].timestamp,
                        symbol: data.symbol.clone(),
                        direction: TradeDirection::Long, // Buy to close short
                        price: candles[i].close,
                        size: 1.0,
                        costs: candles[i].close * 0.001, // 0.1% commission
                    });
                }

                // Open long position
                trades.push(Trade {
                    timestamp: candles[i].timestamp,
                    symbol: data.symbol.clone(),
                    direction: TradeDirection::Long,
                    price: candles[i].close,
                    size: 1.0,
                    costs: candles[i].close * 0.001, // 0.1% commission
                });

                position = Some(TradeDirection::Long);
            } else if cross_below && position != Some(TradeDirection::Short) {
                // Close long position if exists
                if position == Some(TradeDirection::Long) {
                    trades.push(Trade {
                        timestamp: candles[i].timestamp,
                        symbol: data.symbol.clone(),
                        direction: TradeDirection::Short, // Sell to close long
                        price: candles[i].close,
                        size: 1.0,
                        costs: candles[i].close * 0.001, // 0.1% commission
                    });
                }

                // Open short position
                trades.push(Trade {
                    timestamp: candles[i].timestamp,
                    symbol: data.symbol.clone(),
                    direction: TradeDirection::Short,
                    price: candles[i].close,
                    size: 1.0,
                    costs: candles[i].close * 0.001, // 0.1% commission
                });

                position = Some(TradeDirection::Short);
            }
        }

        Ok(trades)
    }
}

// Relative Strength Index (RSI) Strategy
pub struct RSIStrategy {
    pub name: String,
    pub period: usize,
    pub oversold_threshold: f64,
    pub overbought_threshold: f64,
}

impl RSIStrategy {
    pub fn new(period: usize, oversold_threshold: f64, overbought_threshold: f64) -> Self {
        Self {
            name: format!("RSI_{}", period),
            period,
            oversold_threshold,
            overbought_threshold,
        }
    }

    fn calculate_rsi(&self, candles: &[Candle], index: usize) -> Option<f64> {
        if index < self.period || candles.len() <= index {
            return None;
        }

        let mut gain_sum = 0.0;
        let mut loss_sum = 0.0;

        // Calculate initial average gain and loss
        for i in (index - self.period + 1)..=index {
            let price_change = candles[i].close - candles[i - 1].close;
            if price_change >= 0.0 {
                gain_sum += price_change;
            } else {
                loss_sum -= price_change; // Make positive
            }
        }

        let avg_gain = gain_sum / self.period as f64;
        let avg_loss = loss_sum / self.period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        let rsi = 100.0 - (100.0 / (1.0 + rs));

        Some(rsi)
    }
}

impl Strategy for RSIStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, data: &MarketData) -> Result<Vec<Trade>, Box<dyn Error>> {
        let mut trades = Vec::new();
        let candles = &data.candles;

        if candles.len() <= self.period {
            return Ok(trades); // Not enough data
        }

        let mut position: Option<TradeDirection> = None;

        for i in self.period + 1..candles.len() {
            if let Some(rsi) = self.calculate_rsi(candles, i) {
                let prev_rsi = self.calculate_rsi(candles, i - 1).unwrap();

                // Oversold -> Bullish
                if prev_rsi <= self.oversold_threshold && rsi > self.oversold_threshold && position != Some(TradeDirection::Long) {
                    // Close short position if exists
                    if position == Some(TradeDirection::Short) {
                        trades.push(Trade {
                            timestamp: candles[i].timestamp,
                            symbol: data.symbol.clone(),
                            direction: TradeDirection::Long, // Buy to close short
                            price: candles[i].close,
                            size: 1.0,
                            costs: candles[i].close * 0.001,
                        });
                    }

                    // Open long position
                    trades.push(Trade {
                        timestamp: candles[i].timestamp,
                        symbol: data.symbol.clone(),
                        direction: TradeDirection::Long,
                        price: candles[i].close,
                        size: 1.0,
                        costs: candles[i].close * 0.001,
                    });

                    position = Some(TradeDirection::Long);
                }
                // Overbought -> Bearish
                else if prev_rsi >= self.overbought_threshold && rsi < self.overbought_threshold && position != Some(TradeDirection::Short) {
                    // Close long position if exists
                    if position == Some(TradeDirection::Long) {
                        trades.push(Trade {
                            timestamp: candles[i].timestamp,
                            symbol: data.symbol.clone(),
                            direction: TradeDirection::Short, // Sell to close long
                            price: candles[i].close,
                            size: 1.0,
                            costs: candles[i].close * 0.001,
                        });
                    }

                    // Open short position
                    trades.push(Trade {
                        timestamp: candles[i].timestamp,
                        symbol: data.symbol.clone(),
                        direction: TradeDirection::Short,
                        price: candles[i].close,
                        size: 1.0,
                        costs: candles[i].close * 0.001,
                    });

                    position = Some(TradeDirection::Short);
                }
            }
        }

        Ok(trades)
    }
}

// Mean Reversion Strategy
pub struct MeanReversion {
    pub name: String,
    pub period: usize,
    pub std_dev_multiplier: f64,
}

impl MeanReversion {
    pub fn new(period: usize, std_dev_multiplier: f64) -> Self {
        Self {
            name: format!("MeanReversion_{}_{}", period, std_dev_multiplier),
            period,
            std_dev_multiplier,
        }
    }

    fn calculate_bollinger_bands(&self, candles: &[Candle], index: usize) -> Option<(f64, f64, f64)> {
        if index < self.period - 1 || candles.len() <= index {
            return None;
        }

        // Calculate SMA
        let sum: f64 = candles[index - self.period + 1..=index]
            .iter()
            .map(|candle| candle.close)
            .sum();
        let sma = sum / self.period as f64;

        // Calculate standard deviation
        let variance: f64 = candles[index - self.period + 1..=index]
            .iter()
            .map(|candle| (candle.close - sma).powi(2))
            .sum::<f64>() / self.period as f64;
        let std_dev = variance.sqrt();

        // Calculate Bollinger Bands
        let upper_band = sma + (std_dev * self.std_dev_multiplier);
        let lower_band = sma - (std_dev * self.std_dev_multiplier);

        Some((sma, upper_band, lower_band))
    }
}

impl Strategy for MeanReversion {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, data: &MarketData) -> Result<Vec<Trade>, Box<dyn Error>> {
        let mut trades = Vec::new();
        let candles = &data.candles;

        if candles.len() < self.period {
            return Ok(trades); // Not enough data
        }

        let mut position: Option<TradeDirection> = None;

        for i in self.period..candles.len() {
            if let Some((sma, upper_band, lower_band)) = self.calculate_bollinger_bands(candles, i) {
                let close = candles[i].close;

                // Price is below lower band -> Buy
                if close <= lower_band && position != Some(TradeDirection::Long) {
                    // Close short position if exists
                    if position == Some(TradeDirection::Short) {
                        trades.push(Trade {
                            timestamp: candles[i].timestamp,
                            symbol: data.symbol.clone(),
                            direction: TradeDirection::Long, // Buy to close short
                            price: close,
                            size: 1.0,
                            costs: close * 0.001,
                        });
                    }

                    // Open long position
                    trades.push(Trade {
                        timestamp: candles[i].timestamp,
                        symbol: data.symbol.clone(),
                        direction: TradeDirection::Long,
                        price: close,
                        size: 1.0,
                        costs: close * 0.001,
                    });

                    position = Some(TradeDirection::Long);
                }
                // Price is above upper band -> Sell
                else if close >= upper_band && position != Some(TradeDirection::Short) {
                    // Close long position if exists
                    if position == Some(TradeDirection::Long) {
                        trades.push(Trade {
                            timestamp: candles[i].timestamp,
                            symbol: data.symbol.clone(),
                            direction: TradeDirection::Short, // Sell to close long
                            price: close,
                            size: 1.0,
                            costs: close * 0.001,
                        });
                    }

                    // Open short position
                    trades.push(Trade {
                        timestamp: candles[i].timestamp,
                        symbol: data.symbol.clone(),
                        direction: TradeDirection::Short,
                        price: close,
                        size: 1.0,
                        costs: close * 0.001,
                    });

                    position = Some(TradeDirection::Short);
                }
                // Price returns to SMA -> Close position
                else if (position == Some(TradeDirection::Long) && close >= sma) || 
                        (position == Some(TradeDirection::Short) && close <= sma) {
                    
                    trades.push(Trade {
                        timestamp: candles[i].timestamp,
                        symbol: data.symbol.clone(),
                        direction: if position == Some(TradeDirection::Long) { TradeDirection::Short } else { TradeDirection::Long },
                        price: close,
                        size: 1.0,
                        costs: close * 0.001,
                    });

                    position = None;
                }
            }
        }

        Ok(trades)
    }
}

// Factory to create strategies by name
pub fn create_strategy(strategy_name: &str) -> Box<dyn Strategy> {
    match strategy_name {
        "moving_average_crossover" => Box::new(MovingAverageCrossover::new(10, 30)),
        "rsi" => Box::new(RSIStrategy::new(14, 30.0, 70.0)),
        "mean_reversion" => Box::new(MeanReversion::new(20, 2.0)),
        _ => Box::new(MovingAverageCrossover::new(10, 30)), // Default
    }
}