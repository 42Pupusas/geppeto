use base64::{engine::general_purpose, Engine as _};
use futures_util::StreamExt;
use httparse::{Header, Request};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fs;
use tokio_tungstenite::{connect_async_tls_with_config, Connector};
use reqwest::header::{HeaderMap, HeaderValue};

use crate::secrets::{MACAROON_PATH, REST_HOST};


#[derive(Serialize, Debug)]
pub struct InvoiceRequestBody {
    value: u64,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct InvoiceResponseBody {
    pub r_hash: String,
    pub payment_request: String,
}
#[derive(Serialize, Debug, Deserialize)]
pub struct CheckResponseBody {
    pub r_hash: String,
    pub payment_request: String,
    pub settled: bool,
}

pub async fn get_invoice(value:u64) -> Result<InvoiceResponseBody, Box<dyn std::error::Error>> {
    let request_body = InvoiceRequestBody {
        value
    };
    // println!("request_body: {:?}", request_body);
    let mut headers = HeaderMap::new();
    headers.insert(
        "Grpc-Metadata-macaroon",
        HeaderValue::from_str(&hex::encode(fs::read(MACAROON_PATH)?))?,
    );
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .default_headers(headers)
        .build()?;
    let options = client
        .post(&format!("https://{}/v1/invoices", REST_HOST))
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");

    let body = options.text().await.expect("Failed to get response body");
    let invoice_response: InvoiceResponseBody =
        serde_json::from_str(&body).expect("Failed to parse response body");
    Ok(invoice_response)
}

fn generate_random_key() -> String {
    let mut rng = thread_rng();
    let mut key = [0u8; 16];
    rng.fill(&mut key);
    let new_key = general_purpose::STANDARD.encode(key);
    new_key
}

pub async fn stream_invoice_state(r_hash: String) -> Result<(), Box<dyn std::error::Error>> {
    // Format the r-hash into url safe encoded bytes
    let r_hash_bytes = general_purpose::STANDARD.decode(r_hash).unwrap();
    let r_hash_safe = general_purpose::URL_SAFE.encode(r_hash_bytes.clone());

    // Prepare the websocket request URL
    let url = format!(
        "wss://{}/v2/invoices/subscribe/{}?method=GET",
        REST_HOST, r_hash_safe
    );

    // println!("url: {}", url);

    // Prepare the headers
    let macaroon = hex::encode(fs::read(MACAROON_PATH)?);
    let random_key = generate_random_key();
    let mut headers = [
        Header {
            name: "Grpc-Metadata-macaroon",
            value: macaroon.as_bytes(),
        },
        Header {
            name: "Sec-WebSocket-Key",
            value: random_key.as_bytes(),
        },
        Header {
            name: "Host",
            value: REST_HOST.as_bytes(),
        },
        Header {
            name: "Connection",
            value: "Upgrade".as_bytes(),
        },
        Header {
            name: "Upgrade",
            value: "websocket".as_bytes(),
        },
        httparse::Header {
            name: "Sec-WebSocket-Version",
            value: "13".as_bytes(),
        },
    ];
    let mut req = Request::new(&mut headers);
    req.method = Some("GET");
    req.path = Some(&url);
    req.version = Some(1);
    // println!("req: {:?}", req.path);
    // Prepare the websocket connection with SSL
    let danger_conf = Some(tokio_tungstenite::tungstenite::protocol::WebSocketConfig {
        max_send_queue: None,
        max_message_size: None,
        max_frame_size: None,
        accept_unmasked_frames: true,
    });

    let tls_connector = Some(Connector::NativeTls(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()?,
    ));

    // Connect to the lnd server
    let (ws_stream, _) = connect_async_tls_with_config(req, danger_conf, tls_connector).await?;
    let (mut _write, mut read) = ws_stream.split();
    // Wait for relay to send messages back and parse through them
    while let Some(message_result) = read.next().await {
        match message_result {
            Ok(message) => {
                if message.is_text() {
                    let received_text = message.into_text().unwrap();
                    // println!("received text: {}", received_text);
                    let invoice: serde_json::Value = serde_json::from_str(&received_text).unwrap();
                    // println!("invoice: {:?}", invoice);
                    if let Some(settled) = invoice["result"]["settled"].as_bool() {
                        let value = invoice["result"]["value"].as_str().unwrap_or("");
                        println!("Invoice value: {}", value);
                        println!("Invoice settled: {}", settled);
                    }
                }
            }
            Err(e) => {
                println!("Error receiving message: {:?}", e);
            }
        }
    }
    Ok(())
}

pub async fn stream_and_wait_for_invoice_settled(
    r_hash: String,
) -> Result<Option<u64>, Box<dyn std::error::Error>> {
    // Format the r-hash into url safe encoded bytes
    let r_hash_bytes = general_purpose::STANDARD.decode(r_hash).unwrap();
    let r_hash_safe = general_purpose::URL_SAFE.encode(r_hash_bytes.clone());

    // Prepare the websocket request URL
    let url = format!(
        "wss://{}/v2/invoices/subscribe/{}?method=GET",
        REST_HOST, r_hash_safe
    );

    // println!("url: {}", url);

    // Prepare the headers
    let macaroon = hex::encode(fs::read(MACAROON_PATH)?);
    let random_key = generate_random_key();
    let mut headers = [
        httparse::Header {
            name: "Grpc-Metadata-macaroon",
            value: macaroon.as_bytes(),
        },
        httparse::Header {
            name: "Sec-WebSocket-Key",
            value: random_key.as_bytes(),
        },
        httparse::Header {
            name: "Host",
            value: REST_HOST.as_bytes(),
        },
        httparse::Header {
            name: "Connection",
            value: "Upgrade".as_bytes(),
        },
        httparse::Header {
            name: "Upgrade",
            value: "websocket".as_bytes(),
        },
        httparse::Header {
            name: "Sec-WebSocket-Version",
            value: "13".as_bytes(),
        },
    ];
    let mut req = httparse::Request::new(&mut headers);
    req.method = Some("GET");
    req.path = Some(&url);
    req.version = Some(1);

    // Prepare the websocket connection with SSL
    let danger_conf = Some(tokio_tungstenite::tungstenite::protocol::WebSocketConfig {
        max_send_queue: None,
        max_message_size: None,
        max_frame_size: None,
        accept_unmasked_frames: true,
    });

    let tls_connector = Some(Connector::NativeTls(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()?,
    ));

    // Connect to the lnd server
    let (ws_stream, _) = connect_async_tls_with_config(req, danger_conf, tls_connector).await?;
    let (mut _write, mut read) = ws_stream.split();

    // Wait for the invoice to become settled
    while let Some(message_result) = read.next().await {
        match message_result {
            Ok(message) => {
                if message.is_text() {
                    let received_text = message.into_text().unwrap();
                    let invoice: serde_json::Value = serde_json::from_str(&received_text).unwrap();
                    if let Some(settled) = invoice["result"]["settled"].as_bool() {
                        if settled {
                            let value = invoice["result"]["value"].as_str().expect("no value").parse::<u64>().unwrap();
                            return Ok(Some(value)); 
                        } 
                    }
                }
            }
            Err(e) => {
                println!("Error receiving message: {:?}", e);
            }
        }
    }

    // If the loop exited without returning a value, it means the connection was closed
    Ok(None)
}
