use futures_util::{SinkExt, StreamExt};
use serde_json::{from_str, json, Value};
use tokio_tungstenite::connect_async;
use utils::lnd::{get_invoice, stream_and_wait_for_invoice_settled};
use utils::nostr::{create_invoice_event, create_token, NostrSubscription, SignedEvent};
use utils::secrets::RELAY_URL;

#[tokio::main]
async fn main() {
    // Parse URL address into URL struct
    let url = url::Url::parse(RELAY_URL).unwrap();

    // Connect to the server
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to the server");

    // Split the WebSocket into a sender and receiver half
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to token request events
    let prompt_filters = json!({
        "kinds": [29777, 29999],
    });
    write
        .send(NostrSubscription::new(prompt_filters))
        .await
        .unwrap();

    loop {
        // Loop through relay messages
        let msg = read.next().await.unwrap().unwrap().to_string();

        // Handle relay notices
        if let Ok((event_type, _id)) = from_str::<(String, String)>(&msg) {
            println!("event_type: {:?}", event_type);
        }

        // Handle relay events
        if let Ok((_type, _id, event)) = from_str::<(String, String, Value)>(&msg) {
            let nostr_event = SignedEvent::read_new(event);
            match nostr_event.kind {
                29777 => {
                    // Make sure request has a token amount
                    if let Ok(credits_req) = nostr_event.content.parse::<u64>() {
                        println!("credits_req: {:?}", credits_req);
                        let holder = nostr_event.pubkey;
                        let holder_clone = holder.clone();

                        // Prepare an invoice for the requested amount
                        let invoice = get_invoice(credits_req * 10).await.unwrap();
                        let invoice_event = create_invoice_event(holder, invoice.payment_request);

                        // Send the invoice to the requester
                        write
                            .send(invoice_event.prepare_ws_message())
                            .await
                            .unwrap();

                        // Create async task to wait for the invoice to be settled
                        tokio::spawn(async move {

                            // Subscribe to invoice settled events
                            let settled =
                                stream_and_wait_for_invoice_settled(invoice.r_hash.clone())
                                    .await
                                    .unwrap();
                            
                            // We need a new connection as we cannot block the original one
                            let new_connection_url = url::Url::parse(RELAY_URL).unwrap();
                            let (new_connection, _) = connect_async(new_connection_url)
                                .await
                                .expect("Failed to connect");
                            let (mut new_write, mut _new_read) = new_connection.split();

                            // Create and send the requested amount of tokens
                            for _i in 0..settled.unwrap() / 10 {
                                let new_token = create_token(holder_clone.clone());
                                new_write.send(new_token.prepare_ws_message()).await.unwrap();
                            }
                            // Close extra connection when done with requests
                            new_write.close().await.unwrap();
                        });
                    }
                }

                29999 => {
                    println!("Blup");
                }
                _ => {}
            }
        }
    }
}
