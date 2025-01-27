extern crate binance;

use binance::api::*;
use binance::userstream::*;
use binance::websockets::*;
use binance::ws_model::WebsocketEvent;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    //user_stream();
    //user_stream_websocket();
    //market_websocket();
    //kline_websocket();
    //all_trades_websocket();
    last_price();
}

#[allow(dead_code)]
async fn user_stream() {
    let api_key_user = Some("YOUR_API_KEY".into());
    let user_stream: UserStream = Binance::new(api_key_user.clone(), None);

    if let Ok(answer) = user_stream.start().await {
        println!("Data Stream Started ...");
        let listen_key = answer.listen_key;

        match user_stream.keep_alive(&listen_key).await {
            Ok(msg) => println!("Keepalive user data stream: {:?}", msg),
            Err(e) => println!("Error: {}", e),
        }

        match user_stream.close(&listen_key).await {
            Ok(msg) => println!("Close user data stream: {:?}", msg),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("Not able to start an User Stream (Check your API_KEY)");
    }
}

#[allow(dead_code)]
async fn user_stream_websocket() {
    let keep_running = AtomicBool::new(true); // Used to control the event loop
    let api_key_user = Some("YOUR_KEY".into());
    let user_stream: UserStream = Binance::new(api_key_user, None);

    if let Ok(answer) = user_stream.start().await {
        let listen_key = answer.listen_key;

        let mut web_socket: WebSockets<WebsocketEvent> = WebSockets::new(|event: WebsocketEvent| {
            match event {
                WebsocketEvent::AccountUpdate(account_update) => {
                    for balance in &account_update.balances {
                        println!(
                            "Asset: {}, free: {}, locked: {}",
                            balance.asset, balance.free, balance.locked
                        );
                    }
                }
                WebsocketEvent::OrderUpdate(trade) => {
                    println!(
                        "Symbol: {}, Side: {:?}, Price: {}, Execution Type: {:?}",
                        trade.symbol, trade.side, trade.price, trade.execution_type
                    );
                }
                _ => (),
            };

            Ok(())
        });

        web_socket.connect(&listen_key).unwrap(); // check error
        if let Err(e) = web_socket.event_loop(&keep_running) {
            println!("Error: {}", e);
        }
        user_stream.close(&listen_key).await.unwrap();
        web_socket.disconnect().unwrap();
        println!("Userstrem closed and disconnected");
    } else {
        println!("Not able to start an User Stream (Check your API_KEY)");
    }
}

#[allow(dead_code)]
fn market_websocket() {
    let keep_running = AtomicBool::new(true); // Used to control the event loop
    let agg_trade: String = format!("{}@aggTrade", "ethbtc");
    let mut web_socket: WebSockets<WebsocketEvent> = WebSockets::new(|event: WebsocketEvent| {
        match event {
            WebsocketEvent::Trade(trade) => {
                println!("Symbol: {}, price: {}, qty: {}", trade.symbol, trade.price, trade.qty);
            }
            WebsocketEvent::DepthOrderBook(depth_order_book) => {
                println!(
                    "Symbol: {}, Bids: {:?}, Ask: {:?}",
                    depth_order_book.symbol, depth_order_book.bids, depth_order_book.asks
                );
            }
            WebsocketEvent::OrderBook(order_book) => {
                println!(
                    "last_update_id: {}, Bids: {:?}, Ask: {:?}",
                    order_book.last_update_id, order_book.bids, order_book.asks
                );
            }
            _ => (),
        };

        Ok(())
    });

    web_socket.connect(&agg_trade).unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running) {
        println!("Error: {}", e);
    }
    web_socket.disconnect().unwrap();
    println!("disconnected");
}

#[allow(dead_code)]
fn all_trades_websocket() {
    let keep_running = AtomicBool::new(true); // Used to control the event loop
    let agg_trade: String = "!ticker@arr".to_string();
    // NB: you may not ask for both arrays type streams and object type streams at the same time, this holds true in binance connections anyways
    // You cannot connect to multiple things for a single socket
    let mut web_socket: WebSockets<Vec<WebsocketEvent>> = WebSockets::new(|events: Vec<WebsocketEvent>| {
        for tick_events in events {
            if let WebsocketEvent::DayTicker(tick_event) = tick_events {
                println!(
                    "Symbol: {}, price: {}, qty: {}",
                    tick_event.symbol, tick_event.best_bid, tick_event.best_bid_qty
                );
            }
        }

        Ok(())
    });

    web_socket.connect(&agg_trade).unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running) {
        println!("Error: {}", e);
    }
    web_socket.disconnect().unwrap();
    println!("disconnected");
}

#[allow(dead_code)]
fn kline_websocket() {
    let keep_running = AtomicBool::new(true);
    let kline: String = "ethbtc@kline_1m".to_string();
    let mut web_socket: WebSockets<WebsocketEvent> = WebSockets::new(|event: WebsocketEvent| {
        if let WebsocketEvent::Kline(kline_event) = event {
            println!(
                "Symbol: {}, high: {}, low: {}",
                kline_event.kline.symbol, kline_event.kline.low, kline_event.kline.high
            );
        }

        Ok(())
    });

    web_socket.connect(&kline).unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running) {
        println!("Error: {}", e);
    }
    web_socket.disconnect().unwrap();
    println!("disconnected");
}

fn last_price() {
    let keep_running = AtomicBool::new(true);
    let agg_trade: String = "!ticker@arr".to_string();
    let mut btcusdt: f32 = "0".parse().unwrap();

    let mut web_socket: WebSockets<Vec<WebsocketEvent>> = WebSockets::new(|events: Vec<WebsocketEvent>| {
        for tick_events in events {
            if let WebsocketEvent::DayTicker(tick_event) = tick_events {
                if tick_event.symbol == "BTCUSDT" {
                    btcusdt = tick_event.average_price.parse().unwrap();
                    let btcusdt_close: f32 = tick_event.current_close.parse().unwrap();
                    println!("{} - {}", btcusdt, btcusdt_close);

                    if btcusdt_close as i32 == 7000 {
                        // Break the event loop
                        keep_running.store(false, Ordering::Relaxed);
                    }
                }
            }
        }

        Ok(())
    });

    web_socket.connect(&agg_trade).unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running) {
        println!("Error: {}", e);
    }
    web_socket.disconnect().unwrap();
    println!("disconnected");
}
