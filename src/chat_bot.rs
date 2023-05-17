use futures_util::{SinkExt, StreamExt};
use serde_json::{from_str, json, Value};
use tokio_tungstenite::connect_async;
use utils::gpt::chat_api_request;
use utils::nostr::{check_for_token, create_prompt_event, delete_token, SignedEvent, NostrSubscription};
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

    // Subscribe to prompt events
    let prompt_filters = json!({
        "kinds": [29000, 29999],
    });
  
    // Send the subscription event
    write
        .send(NostrSubscription::new(prompt_filters))
        .await
        .expect("Failed to send JSON");


      loop {
        // Parse messages from relay
        let msg = read.next().await.unwrap().unwrap().to_string();

        // Handle relay notices
        if let Ok((event_type, _id)) = from_str::<(String, String)>(&msg) {
            println!("event_type: {:?}", event_type);
        }

        // Handle relay events
        if let Ok((_type, _id, event)) = from_str::<(String, String, Value)>(&msg) {
            let nostr_event = SignedEvent::read_new(event);
            match nostr_event.kind {
                29000 => {
                    // Check if the user has a token
                    match check_for_token(nostr_event.pubkey).await {
                        Ok(token) => {
                            tokio::spawn(async move {
                                
                                // Get response from GPT-3 
                                let chat_response =
                                    chat_api_request(nostr_event.content).await.unwrap();
                                let prompt_event = create_prompt_event(chat_response);
                                
                                // We need a new connection as we cannot block the original one
                                let url = url::Url::parse(RELAY_URL).unwrap();

                                // Connect to the server
                                let (ws_stream, _) =
                                    connect_async(url).await.expect("Failed to connect");

                                // Split the WebSocket into a sender and receiver half
                                let (mut new_write, mut _new_read) = ws_stream.split();
                                
                                // Respond to prompt
                                new_write
                                    .send(prompt_event.prepare_ws_message())
                                    .await
                                    .unwrap();
                                
                                // Delete a token
                                let token_deletion = delete_token(token);
                                new_write
                                    .send(token_deletion.prepare_ws_message())
                                    .await
                                    .unwrap();
                                new_write.close().await.unwrap();
                            });
                        }

                        // Respond if the user doesn't have a token
                        Err(_) => {
                            tokio::spawn(async move {
                                let prompt_response =
                                    create_prompt_event("No token found".to_string());
                                let url = url::Url::parse(RELAY_URL).unwrap();
                                let (ws_stream, _) =
                                    connect_async(url).await.expect("Failed to connect");
                                let (mut new_write, mut _new_read) = ws_stream.split();
                                new_write
                                    .send(prompt_response.prepare_ws_message())
                                    .await
                                    .unwrap();
                                new_write.close().await.unwrap();

                                println!("No token found for holder");
                            });
                        }
                    }
                }

                29999 => {
                    println!("Blop");
                }

                _ => {}
            }
        }
    }
}
