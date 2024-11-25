use futures::{SinkExt, StreamExt, TryFutureExt};
use std::io::Write;
use tokio::sync::mpsc;
use warp::ws::Message;
use warp::Filter;

mod nal_unit_parser;

#[tokio::main]
async fn main() {
    let web_route = warp::fs::dir("static/www");
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(handle_connection));

    let routes = web_route.or(ws_route);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_connection(ws: warp::ws::WebSocket) {
    // Split web sockets into a transmitter (write) and a receiver (read), which allows us to handle
    // writing and reading independently. This is achieved by the split method provided by the
    // Stream and Sink traits via the futures crate.
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::channel::<String>(32);

    let mut parser = nal_unit_parser::NalUnitParser::new();

    tokio::task::spawn(async move {
        while let Some(nal_unit_string) = rx.recv().await {
            ws_tx
                .send(Message::text(nal_unit_string))
                .unwrap_or_else(|e| eprintln!("Error: {}", e))
                .await
        }
    });

    while let Some(Ok(msg)) = ws_rx.next().await {
        let _ = parser.write(msg.as_bytes());
        for nal_unit in nal_units {
            (tx.send(nal_unit.to_string()).await).unwrap_or_else(|e| eprintln!("Error: {}", e));
        }
    }
}
