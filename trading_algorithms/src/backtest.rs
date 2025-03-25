use crate::data;
use crate::models::{BacktestResult, Trade, TradeDirection};
use crate::strategies;
use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc};

pub async fn run_backtest(strategy_name: &str) -> Result<BacktestResult, Box<dyn Error>> {
    // Fetch historical data for backtesting
    let symbol = "BTC/USD";
    let start_date = "2023-01-01";
    let end_date = "2023-12-31";
    
    println!("Fetching data for {} from {} to {}", symbol, start_date, end_date);
    let market_data = data::fetch_historical_data(symbol, start_date, end_date).await?;
    
    // Create strategy
    let strategy = strategies::create_strategy(strategy_name);
    println!("Running strategy: {}", strategy.name());
    
    // Execute strategy on historical data
    let trades = strategy.execute(&market_data)?;
    println!("Generated {} trades", trades.len());
    
    // Calculate performance metrics
    let (total_profit_loss, winning_trades, losing_trades) = calculate_profit_loss(&trades);
    let equity_curve = generate_equity_curve(&trades);
    let sharpe_ratio = calculate_sharpe_ratio(&equity_curve);
    let max_drawdown = calculate_max_drawdown(&equity_curve);
    
    // Additional metrics
    let mut metrics = HashMap::new();
    metrics.insert("win_rate".to_string(), winning_trades as f64 / trades.len() as f64);
    metrics.insert("avg_trade_profit".to_string(), total_profit_loss / trades.len() as f64);
    
    Ok(BacktestResult {
        strategy_name: strategy.name().to_string(),
        total_trades: trades.len(),
        winning_trades,
        losing_trades,
        total_profit_loss,
        sharpe_ratio,
        max_drawdown,
        trades,
        equity_curve,
        metrics,
    })
}

fn calculate_profit_loss(trades: &[Trade]) -> (f64, usize, usize) {
    let mut total_profit_loss = 0.0;
    let mut winning_trades = 0;
    let mut losing_trades = 0;
    
    // In a real system, we would match opening and closing trades
    // This is a simplified version that assumes alternating buy/sell
    
    let mut position: Option<(f64, f64)> = None; // (price, size)
    
    for trade in trades {
        match trade.direction {
            TradeDirection::Long => {
                // Opening a long position or closing a short position
                if let Some((entry_price, size)) = position {
                    // Closing a short position
                    let profit_loss = (entry_price - trade.price) * size - trade.costs;
                    total_profit_loss += profit_loss;
                    
                    if profit_loss > 0.0 {
                        winning_trades += 1;
                    } else {
                        losing_trades += 1;
                    }
                    
                    position = None;
                } else {
                    // Opening a long position
                    position = Some((trade.price, trade.size));
                }
            }
            TradeDirection::Short => {
                // Opening a short position or closing a long position
                if let Some((entry_price, size)) = position {
                    // Closing a long position
                    let profit_loss = (trade.price - entry_price) * size - trade.costs;
                    total_profit_loss += profit_loss;
                    
                    if profit_loss > 0.0 {
                        winning_trades += 1;
                    } else {
                        losing_trades += 1;
                    }
                    
                    position = None;
                } else {
                    // Opening a short position
                    position = Some((trade.price, trade.size));
                }
            }
        }
    }
    
    (total_profit_loss, winning_trades, losing_trades)
}

fn generate_equity_curve(trades: &[Trade]) -> Vec<(DateTime<Utc>, f64)> {
    let mut equity_curve = Vec::new();
    let mut equity = 10000.0; // Starting capital
    
    // Add initial point
    if !trades.is_empty() {
        equity_curve.push((trades[0].timestamp, equity));
    }
    
    let mut position: Option<(f64, f64)> = None; // (price, size)
    
    for trade in trades {
        match trade.direction {
            TradeDirection::Long => {
                // Opening a long position or closing a short position
                if let Some((entry_price, size)) = position {
                    // Closing a short position
                    let profit_loss = (entry_price - trade.price) * size - trade.costs;
                    equity += profit_loss;
                    position = None;
                } else {
                    // Opening a long position
                    position = Some((trade.price, trade.size));
                    equity -= trade.costs; // Subtract trading costs
                }
            }
            TradeDirection::Short => {
                // Opening a short position or closing a long position
                if let Some((entry_price, size)) = position {
                    // Closing a long position
                    let profit_loss = (trade.price - entry_price) * size - trade.costs;
                    equity += profit_loss;
                    position = None;
                } else {
                    // Opening a short position
                    position = Some((trade.price, trade.size));
                    equity -= trade.costs; // Subtract trading costs
                }
            }
        }
        
        // Record equity at each trade
        equity_curve.push((trade.timestamp, equity));
    }
    
    equity_curve
}

fn calculate_sharpe_ratio(equity_curve: &[(DateTime<Utc>, f64)]) -> f64 {
    if equity_curve.len() < 2 {
        return 0.0;
    }
    
    // Calculate daily returns
    let mut returns = Vec::new();
    for i in 1..equity_curve.len() {
        let daily_return = (equity_curve[i].1 - equity_curve[i-1].1) / equity_curve[i-1].1;
        returns.push(daily_return);
    }
    
    // Calculate average return
    let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;
    
    // Calculate standard deviation of returns
    let variance = returns.iter()
        .map(|r| (r - avg_return).powi(2))
        .sum::<f64>() / returns.len() as f64;
    let std_dev = variance.sqrt();
    
    // Calculate annualized Sharpe ratio (assuming 252 trading days)
    // and risk-free rate of 0% for simplicity
    if std_dev == 0.0 {
        return 0.0;
    }
    
    (avg_return / std_dev) * (252.0_f64).sqrt()
}

fn calculate_max_drawdown(equity_curve: &[(DateTime<Utc>, f64)]) -> f64 {
    if equity_curve.len() < 2 {
        return 0.0;
    }
    
    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0].1;
    
    for (_, equity) in equity_curve {
        if *equity > peak {
            peak = *equity;
        } else {
            let drawdown = (peak - *equity) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
    }
    
    max_drawdown
}