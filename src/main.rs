use futures::{SinkExt, StreamExt};
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

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

async fn handle_connection(ws: warp::ws::WebSocket) {
    // Split web sockets into a transmitter (write) and a receiver (read), which allows us to handle
    // writing and reading independently. This is achieved by the split method provided by the
    // Stream and Sink traits via the futures crate.
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::channel::<String>(32);

    let mut nal_unit_count = nal_unit_parser::NalUnitCount::new();

    // This spawns a task which allows us to send proccessed messages to the WebSocket.
    tokio::task::spawn(async move {
        // while let Some(unit) = parser.next().await {
        //      nal_unit_count.update(unit);
        //      if ws_tx.send(Message::text(msg)).await.is_err() {
        //          break;
        //      }
        // }
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = ws_rx.next().await {
        // let mut parser = parser.lock();
        // parser.write_all(msg.as_bytes());
        process_bytes(msg.as_bytes(), &mut nal_unit_count, &tx).await
    }
}

async fn process_bytes(
    bytes: &[u8],
    nal_unit_count: &mut nal_unit_parser::NalUnitCount,
    tx: &mpsc::Sender<String>,
) {
    for nal_unit in nal_unit_parser::parse(bytes) {
        nal_unit_count.update(&nal_unit);

        if (tx.send(nal_unit_count.format()).await).is_err() {
            break;
        }
    }
}
