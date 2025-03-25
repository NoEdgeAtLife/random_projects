mod data;
mod strategies;
mod backtest;
mod utils;
mod models;
mod execution;

use std::error::Error;
use models::TradeDirection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Trading Algorithms Platform");
    println!("==========================");
    
    // Example 1: Run a backtest with a trading strategy
    let strategy_name = "moving_average_crossover";
    println!("Running backtest for {} strategy", strategy_name);
    
    let result = backtest::run_backtest(strategy_name).await?;
    println!("Backtest results:");
    println!("  Strategy: {}", result.strategy_name);
    println!("  Total trades: {}", result.total_trades);
    println!("  Win rate: {:.2}%", result.metrics.get("win_rate").unwrap_or(&0.0) * 100.0);
    println!("  Total P&L: ${:.2}", result.total_profit_loss);
    println!("  Sharpe ratio: {:.2}", result.sharpe_ratio);
    println!("  Max drawdown: {:.2}%", result.max_drawdown * 100.0);
    
    // Example 2: Demonstrate execution algorithms
    println!("\nDemonstrating Execution Algorithms");
    println!("================================");
    
    // Fetch sample market data
    let symbol = "ETH/USD";
    let start_date = "2023-01-01";
    let end_date = "2023-01-31";
    
    println!("Fetching data for {} from {} to {}", symbol, start_date, end_date);
    let market_data = data::fetch_historical_data(symbol, start_date, end_date).await?;
    
    // Set up execution parameters
    let order_size = 100.0;
    let direction = TradeDirection::Long;
    let start_time = market_data.candles.first().unwrap().timestamp;
    let end_time = market_data.candles.last().unwrap().timestamp;
    
    // Run VWAP execution
    let vwap = execution::create_execution_algorithm("vwap");
    println!("\nRunning {} execution algorithm", vwap.name());
    let vwap_trades = vwap.execute(&market_data, order_size, direction, start_time, Some(end_time))?;
    
    let vwap_executed = vwap_trades.iter().map(|t| t.size).sum::<f64>();
    let vwap_avg_price = vwap_trades.iter().map(|t| t.price * t.size).sum::<f64>() / vwap_executed;
    let vwap_total_cost = vwap_trades.iter().map(|t| t.costs).sum::<f64>();
    
    println!("  VWAP executed: {:.2} units at average price ${:.2}", vwap_executed, vwap_avg_price);
    println!("  Total trades: {}", vwap_trades.len());
    println!("  Total cost: ${:.2}", vwap_total_cost);
    
    // Run TWAP execution
    let twap = execution::create_execution_algorithm("twap");
    println!("\nRunning {} execution algorithm", twap.name());
    let twap_trades = twap.execute(&market_data, order_size, direction, start_time, Some(end_time))?;
    
    let twap_executed = twap_trades.iter().map(|t| t.size).sum::<f64>();
    let twap_avg_price = twap_trades.iter().map(|t| t.price * t.size).sum::<f64>() / twap_executed;
    let twap_total_cost = twap_trades.iter().map(|t| t.costs).sum::<f64>();
    
    println!("  TWAP executed: {:.2} units at average price ${:.2}", twap_executed, twap_avg_price);
    println!("  Total trades: {}", twap_trades.len());
    println!("  Total cost: ${:.2}", twap_total_cost);
    
    // Run Implementation Shortfall execution
    let is = execution::create_execution_algorithm("is");
    println!("\nRunning {} execution algorithm", is.name());
    let is_trades = is.execute(&market_data, order_size, direction, start_time, Some(end_time))?;
    
    let is_executed = is_trades.iter().map(|t| t.size).sum::<f64>();
    let is_avg_price = is_trades.iter().map(|t| t.price * t.size).sum::<f64>() / is_executed;
    let is_total_cost = is_trades.iter().map(|t| t.costs).sum::<f64>();
    
    println!("  IS executed: {:.2} units at average price ${:.2}", is_executed, is_avg_price);
    println!("  Total trades: {}", is_trades.len());
    println!("  Total cost: ${:.2}", is_total_cost);
    
    // Run Adaptive Market Execution
    let adaptive = execution::create_execution_algorithm("adaptive");
    println!("\nRunning {} execution algorithm", adaptive.name());
    let adaptive_trades = adaptive.execute(&market_data, order_size, direction, start_time, Some(end_time))?;
    
    let adaptive_executed = adaptive_trades.iter().map(|t| t.size).sum::<f64>();
    let adaptive_avg_price = adaptive_trades.iter().map(|t| t.price * t.size).sum::<f64>() / adaptive_executed;
    let adaptive_total_cost = adaptive_trades.iter().map(|t| t.costs).sum::<f64>();
    
    println!("  Adaptive executed: {:.2} units at average price ${:.2}", adaptive_executed, adaptive_avg_price);
    println!("  Total trades: {}", adaptive_trades.len());
    println!("  Total cost: ${:.2}", adaptive_total_cost);
    
    // Compare execution performance
    println!("\nExecution Performance Comparison");
    println!("===============================");
    println!("Algorithm      | Avg Price    | # Trades | Total Cost");
    println!("-------------- | ------------ | -------- | ----------");
    println!("{:<14} | ${:<11.2} | {:<8} | ${:.2}", vwap.name(), vwap_avg_price, vwap_trades.len(), vwap_total_cost);
    println!("{:<14} | ${:<11.2} | {:<8} | ${:.2}", twap.name(), twap_avg_price, twap_trades.len(), twap_total_cost);
    println!("{:<14} | ${:<11.2} | {:<8} | ${:.2}", is.name(), is_avg_price, is_trades.len(), is_total_cost);
    println!("{:<14} | ${:<11.2} | {:<8} | ${:.2}", adaptive.name(), adaptive_avg_price, adaptive_trades.len(), adaptive_total_cost);
    
    Ok(())
}
