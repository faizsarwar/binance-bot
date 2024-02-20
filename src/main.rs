use binance::account::*;
use binance::api::Binance;
use binance::futures::account::*;
use binance::futures::market::*;
use binance::market::*;
use std::io;
fn main() {
    // Replace with your Binance API key and secret
    let api_key = Some("YOUR_API_KEY".into());
    let secret_key = Some("YOUR_SECRET_KEY".into());

    let client: Account = Binance::new(api_key.clone(), secret_key.clone());
    let futures_client: FuturesAccount = Binance::new(api_key, secret_key);

    // Define symbols
    let spot_symbol = "BTCUSDT";
    let futures_symbol = "BTCUSDT";

    // define condition
    let min_order_size: f64 = 25.0;
    let max_order_size: f64 = 100.0;

    // Prompt the user for input
    println!("Please Enter Your position in USD:       ");

    // Read the input as a string
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    
    // Parse the input into an f64
    let input_amount: f64 = input.trim().parse().unwrap();

    loop {
        // fetch Data
        let spot_bid_price = get_spot_bid_price(&spot_symbol);
        let futures_bid_size = get_futures_bid_size(&futures_symbol);

        // print fetched Data
        println!("Fetch Spot Bid Price: {} ", spot_bid_price);
        println!("Fetch Future Bid Size: {}", futures_bid_size);
     
        if futures_bid_size < min_order_size && futures_bid_size < max_order_size  && futures_bid_size < input_amount{

            let mut buy_prices_sum = 0.0;
            let mut sell_prices_sum = 0.0;

            // Execute spot limit buy order
            let spot_order = client
                .limit_buy(spot_symbol, futures_bid_size, spot_bid_price)
                .unwrap();

            // update buy_prices_sum
            buy_prices_sum = buy_prices_sum + spot_bid_price;

            println!(
                "Order{}: LIMIT BUY {} {}",
                spot_order.order_id, futures_bid_size, spot_symbol
            );
            println!(
                "Order{}: PARTIALLY FILLED {} {} ",
                spot_order.order_id, spot_order.executed_qty, spot_symbol
            );

            // Execute futures market sell order
            let futures_order = futures_client
                .market_sell(futures_symbol, spot_order.executed_qty)
                .unwrap();

            // update sell_prices_sum
            sell_prices_sum = sell_prices_sum + futures_order.avg_price;

            println!(
                "Order{}: MARKET SELL {} {}_PREP",
                futures_order.order_id, spot_order.executed_qty, futures_symbol
            );

            // check if spot is completely filled or not
            let mut is_spot_filled = spot_order.orig_qty == spot_order.executed_qty;

            while !is_spot_filled {
                // track order execution later
                let limit_order_id = spot_order.order_id;
                let limit_order_status = client.order_status(spot_symbol, limit_order_id).unwrap();

                // if the order is completly filled
                if limit_order_status.orig_qty == limit_order_status.executed_qty {
                    println!(
                        "Order{}:  FILLED {} {} ",
                        spot_order.order_id, limit_order_status.executed_qty, spot_symbol
                    );

                    // Execute futures market sell order for remaining quantity
                    let remaining_qty = spot_order.orig_qty - spot_order.executed_qty;
                    let futures_order = futures_client
                        .market_sell(futures_symbol, remaining_qty)
                        .unwrap();

                    // update sell_prices_sum
                    sell_prices_sum = sell_prices_sum + futures_order.avg_price;

                    println!(
                        "Order{}: MARKET SELL {} {}_PREP",
                        futures_order.order_id, remaining_qty, futures_symbol
                    );

                    is_spot_filled = true
                }

            }

            // Calculate spread on completion
            let spread_bps = calculate_spread_bps(buy_prices_sum, sell_prices_sum, futures_bid_size);
            println!("Spread on completion: {} bps", spread_bps);
        }
    }
}

// need to return price
fn get_spot_bid_price(symbol: &str) -> f64 {
    let market: Market = Binance::new(None, None);
    let book_ticker = market.get_book_ticker(symbol).unwrap();
    book_ticker.bid_price
}

// need to return amount of futures
fn get_futures_bid_size(symbol: &str) -> f64 {
    let market: FuturesMarket = Binance::new(None, None);
    let book_ticker = market.get_book_ticker(symbol).unwrap();
    book_ticker.bid_qty
}

// calculate spread bps
fn calculate_spread_bps(buy_prices_sum: f64, sell_prices_sum: f64, total_quantity_executed: f64 ) -> f64{

    let weighted_avg_buy_price = ( buy_prices_sum * total_quantity_executed ) / total_quantity_executed;

    let weighted_avg_sell_price = ( sell_prices_sum * total_quantity_executed ) / total_quantity_executed;

    let spread_bps = ( ( weighted_avg_sell_price - weighted_avg_buy_price  ) / weighted_avg_buy_price ) * 10000.0;

    spread_bps
}
