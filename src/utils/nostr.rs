use futures_util::{SinkExt, StreamExt};
use rand::{thread_rng, Rng};
use secp256k1::{KeyPair, Message, PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, to_value, Value};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

use crate::secrets::{PRIV_KEY, PUB_KEY};

#[derive(Serialize, Deserialize)]
pub struct NostrSubscription {
    id: String,
    filters: Value,
}

impl NostrSubscription {
    pub fn new(filter: Value) -> WsMessage {
        let id = hex::encode(&new_keys()[..]);
        let nostr_subscription = NostrSubscription {
            id,
            filters: filter,
        };
        let nostr_subscription_string = WsMessage::Text(json!(["REQ", nostr_subscription.id, nostr_subscription.filters]).to_string());
        nostr_subscription_string
    }
}
#[derive(Serialize, Deserialize)]
pub struct Event {
    pubkey: String,
    created_at: u64,
    kind: u32,
    tags: Vec<Vec<String>>,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedEvent {
    pub id: String,
    pub pubkey: String,
    pub created_at: u64,
    pub kind: u32,
    pub tags: Vec<Vec<String>>,
    pub content: String,
    pub sig: String,
}

impl Event {
    pub fn sign_event(&self, keypair: KeyPair) -> SignedEvent {
        let value = to_value(&self).unwrap();

        let json_str = json!([
            0,
            value["pubkey"],
            value["created_at"],
            value["kind"],
            value["tags"],
            value["content"]
        ]);
        // Serialize the event as JSON

        // Compute the SHA256 hash of the serialized JSON string
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(&json_str).unwrap());
        let hash_result = hasher.finalize();
        let event_id = hex::encode(hash_result);
        println!("event_id: {:?}", event_id);
        let secp = Secp256k1::new();
        let id_message = Message::from_slice(&hash_result).unwrap();
        let signature = secp.sign_schnorr_no_aux_rand(&id_message, &keypair);

        let signed_event = SignedEvent {
            id: event_id,
            pubkey: self.pubkey.clone(),
            created_at: self.created_at,
            kind: self.kind,
            tags: self.tags.clone(),
            content: self.content.clone(),
            sig: signature.to_string(),
        };

        signed_event
    }
}

impl SignedEvent {
    pub fn read_new(json: Value) -> SignedEvent {
        let event: SignedEvent = serde_json::from_value(json).unwrap();
        event
    }

    pub fn prepare_ws_message(&self) -> WsMessage {
        let event_string = json!(["EVENT", self]).to_string();
        let event_ws_message = WsMessage::Text(event_string);
        event_ws_message
    }
}

pub fn new_keys() -> SecretKey {
    let mut rng = thread_rng();

    // Generate a random 256-bit integer as the private key
    let private_key: [u8; 32] = rng.gen();

    // Convert the private key to a secp256k1 SecretKey object
    let secret_key = SecretKey::from_slice(&private_key).unwrap();

    // Return the private key in hexadecimal format
    secret_key
}

pub fn get_public_key(private_key: &SecretKey) -> PublicKey {
    // Create a secp256k1 context
    let secp = Secp256k1::new();

    // Generate the public key from the private key
    let public_key = PublicKey::from_secret_key(&secp, private_key);

    public_key
}

pub fn get_unix_timestamp() -> u64 {
    // Get the current time as a SystemTime object
    let current_time = SystemTime::now();

    // Get the duration between the current time and the Unix epoch
    let duration_since_epoch = current_time.duration_since(UNIX_EPOCH).unwrap();

    // Get the number of seconds since the Unix epoch as a u64 value
    let unix_timestamp = duration_since_epoch.as_secs();

    unix_timestamp
}

pub fn create_token(holder: String) -> SignedEvent {
    
    // Random number for token id
    let token_id = hex::encode(&new_keys()[..]);

    // Keypair for the server bot
    let keypair = KeyPair::from_seckey_str(&Secp256k1::new(), PRIV_KEY).unwrap();
    let public_key = get_public_key(&keypair.secret_key());
    
    // Values for nostr event, tagging token holder and adding token id as content
    let created_at = get_unix_timestamp();
    let kind = 9777;
    let tags: Vec<Vec<String>> = vec![vec!["p".to_string(), holder.clone()]];
    let content = token_id;

    let my_event = Event {
        pubkey: public_key.to_string()[2..].to_string(),
        created_at,
        kind,
        tags,
        content,
    };
    let signed_event = my_event.sign_event(keypair); 
    signed_event
    
}

pub fn delete_token(event: String) -> SignedEvent {
    
    // Get the server bot keypair
    let keypair = KeyPair::from_seckey_str(&Secp256k1::new(), PRIV_KEY).unwrap();
    let public_key = get_public_key(&keypair.secret_key());
    
    // Values for the nostr event, tagging the token even id to delete 
    let created_at = get_unix_timestamp();
    let kind = 5;
    let tags = vec![vec!["e".to_string(), event]];
    let content = "spent token".to_string();

    let my_event = Event {
        pubkey: public_key.to_string()[2..].to_string(),
        created_at,
        kind,
        tags,
        content: content.to_string(),
    };
    
    let signed_event = my_event.sign_event(keypair);
    signed_event
    // let event_string = serde_json::to_string(&signed_event).unwrap();
    // event_string
}

pub fn create_prompt_event(content: String) -> SignedEvent {
    
    // Get the server bot keypair 
    let keypair = KeyPair::from_seckey_str(&Secp256k1::new(), PRIV_KEY).unwrap();
    let public_key = get_public_key(&keypair.secret_key());
    
    // Get event values
    let created_at = get_unix_timestamp();
    let kind = 29001;
    let tags: Vec<Vec<String>> = vec![];

    let my_event = Event {
        pubkey: public_key.to_string()[2..].to_string(),
        created_at,
        kind,
        tags,
        content,
    };
    let signed_event = my_event.sign_event(keypair);
    signed_event

}

pub async fn check_for_token(holder: String) -> Result<String, bool> {
    // Parse URL address into URL struct
    let url = url::Url::parse("ws://192.168.1.5:6969").unwrap();

    // Connect to the server
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Looking for token...");

    // Split the WebSocket into a sender and receiver half
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to prompt events
    let prompt_filters = json!({
        "kinds": [9777],
        "authors": [PUB_KEY],
        "limit": 1,
        "#p": [holder]
    });

    // Send the subscription event
    write
        .send(NostrSubscription::new(prompt_filters))
        .await
        .expect("Failed to send JSON");

    loop {
        // Parse messages from relay
        let msg = read.next().await.unwrap().unwrap().to_string();

        //Handle relay notices
        if let Ok((event_type, _id)) = from_str::<(String, String)>(&msg) {
            match event_type.as_str() {
                "EOSE" => {
                    println!("Token not found!");
                    return Err(false);
                }
                _ => {}
            }
        }

        if let Ok((_type, _id, event)) = from_str::<(String, String, Value)>(&msg) {
            match event["kind"].as_i64().unwrap() {
                9777 => {
                    return Ok(event["id"].as_str().unwrap().to_string());
                }

                _ => {}
            }
        }
    }
}

pub fn test_event(content: String, kind: u32) -> SignedEvent {
    let private_key = new_keys();
    let public_key = get_public_key(&private_key);
    let keypair = KeyPair::from_secret_key(&Secp256k1::new(), &private_key);
    // Example values for the event fields
    let created_at = get_unix_timestamp();
    let kind = kind;
    let tags: Vec<Vec<String>> = vec![];
    let content = content;

    let my_event = Event {
        pubkey: public_key.to_string()[2..].to_string(),
        created_at,
        kind,
        tags,
        content: content.to_string(),
    };
    let signed_event = my_event.sign_event(keypair);
    signed_event
    // let event_string = serde_json::to_string(&signed_event).unwrap();
    // event_string
}

pub fn create_invoice_event(holder: String, invoice: String) -> SignedEvent {
    let keypair = KeyPair::from_seckey_str(&Secp256k1::new(), PRIV_KEY).unwrap();
    let public_key = get_public_key(&keypair.secret_key());
    // Example values for the event fields
    let created_at = get_unix_timestamp();
    let kind = 29778;
    let tags: Vec<Vec<String>> = vec![vec!["p".to_string(), holder.clone()]];
    let content = invoice;

    let my_event = Event {
        pubkey: public_key.to_string()[2..].to_string(),
        created_at,
        kind,
        tags,
        content,
    };

    let signed_event = my_event.sign_event(keypair);
    signed_event
}
