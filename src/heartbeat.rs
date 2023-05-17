use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use utils::nostr::heartbeat_event;
use utils::secrets::RELAY_URL;


#[tokio::main]
async fn main() {
    // Parse URL address into URL struct
    let url = url::Url::parse(RELAY_URL).unwrap();

    // Connect to the server
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to the server");

    // Split the WebSocket into a sender and receiver half
    let (mut write, mut _read) = ws_stream.split();

    loop {
        let heartbeat = heartbeat_event();
        write
            .send(heartbeat.prepare_ws_message())
            .await
            .expect("Failed to send JSON");
        tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        println!("Blup");
    }
}