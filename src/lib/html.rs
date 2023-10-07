use askama::Template;
use axum::{
    debug_handler,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use chrono::NaiveDateTime;
use nostro2::{
    notes::SignedNote,
    relays::{NostrRelay, RelayEvents}, utils::get_unix_timestamp,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value, from_str};
use tracing::log::info;

#[derive(Template)]
#[template(path = "home.html")]
struct HomepageTemplate;

pub async fn homepage() -> impl IntoResponse {
    HtmlTemplate(HomepageTemplate {})
}

#[derive(Template)]
#[template(path = "checkIn.html")]
struct CheckInTemplate;

pub async fn check_in() -> impl IntoResponse {
    HtmlTemplate(CheckInTemplate {})
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate;

pub async fn about() -> impl IntoResponse {
    HtmlTemplate(AboutTemplate {})
}

#[derive(Template)]
#[template(path = "keys.html")]
struct KeysTemplate;

pub async fn keys() -> impl IntoResponse {
    HtmlTemplate(KeysTemplate {})
}

#[derive(Template)]
#[template(path = "navigator.html")]
struct NavigatorTemplate {
    pub pubkey: String,
}

pub async fn navigator(user_note: Json<SignedNote>) -> impl IntoResponse {
    info!("User checked in: {}", user_note.get_pubkey());
    if user_note.verify_signature() {
        info!("User signature verified");
        HtmlTemplate(NavigatorTemplate {
            pubkey: user_note.get_pubkey().to_string(),
        })
    } else {
        info!("User signature failed to verify");
        HtmlTemplate(NavigatorTemplate {
            pubkey: "Failed to verify signature".to_string(),
        })
    }
}

#[derive(Template)]
#[template(path = "visions.html")]
pub struct VisionTemplate {
    pub visions: Vec<Vision>,
}

#[derive(Deserialize, Debug)]
pub struct Vision {
    pub author: String,
    pub content: String,
    pub timestamp: u64,
    pub id: String,
}

impl Vision {
    pub fn get_date_time(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp_opt(self.timestamp as i64, 0).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VisionFilter {
    pub relay: String,
    pub authors: String,
    pub kinds: String,
    pub until: String,
    pub ids: String,
}

impl VisionFilter {
    pub fn build_filter(&self) -> Value {

        let mut nostr_filter = json!({
            "limit": 42,
        });
        let kinds = str::parse::<u32>(&self.kinds);
        let until = str::parse::<u64>(&self.until);
        if let Ok(kinds) = kinds {
            nostr_filter["kinds"] = json!([kinds]);
        }
        if let Ok(until) = until {
            let until = get_unix_timestamp() - until;
            nostr_filter["until"] = json!(until);
        }
        if self.authors != "" {
            nostr_filter["authors"] = json!([self.authors]);
        }

        if self.ids != "" {
            nostr_filter["ids"] = json!([self.ids]);
        }
        info!("Filter: {}", nostr_filter);
        nostr_filter
    }
}

pub async fn visions(filter_request: Json<VisionFilter>) -> impl IntoResponse {
    if let Ok(relay) = NostrRelay::new(&filter_request.relay).await {
        info!("Connected to relay: {}", filter_request.relay);

        let mut visions = vec![];
        let _ = relay.subscribe(filter_request.build_filter()).await;
        
        while let Some(Ok(relay_message)) = relay.read_from_relay().await {
            match relay_message {
                RelayEvents::EVENT(_, _, note) => {
                    info!("Got note: {}", note.get_id());
                    let new_vision = Vision {
                        author: note.get_pubkey().to_string(),
                        content: note.get_content().to_string(),
                        timestamp: note.get_created_at(),
                        id: note.get_id().to_string(),
                    };
                    visions.push(new_vision);
                }
                RelayEvents::EOSE(..) => {
                    info!("End of stream");
                    let _ = relay.close().await;
                }
                _ => {}
            }
        }
        HtmlTemplate(VisionTemplate { visions })
    } else {
        HtmlTemplate(VisionTemplate { visions: vec![] })
    }
}

#[derive(Template)]
#[template(path = "didSend.html")]
pub struct DidSendTemplate {
    pub sent: bool,
}

#[derive(Deserialize, Debug)]
pub struct SendNoteRequest {
    pub note: String,
    pub relay: String,
}

#[debug_handler]
pub async fn send_note(note_request: Json<SendNoteRequest>) -> impl IntoResponse {
    let signed_note = from_str::<SignedNote>(&note_request.note).unwrap();
    if signed_note.verify_signature() && signed_note.verify_content() {
        info!("Note signature verified");
        if let Ok(relay) = NostrRelay::new(&note_request.relay).await {
            info!("Connected to relay");
            let _ = relay.send_note(signed_note).await;
            info!("Sent note");
            let _ = relay.close().await;
            HtmlTemplate(DidSendTemplate { sent: true })
        } else {
            HtmlTemplate(DidSendTemplate { sent: false })
        }
    } else {
        info!("Note signature failed to verify");
        HtmlTemplate(DidSendTemplate { sent: false })
    }

}

pub struct HtmlTemplate<T>(T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),

            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed To Template HTML: {}", e),
            )
                .into_response(),
        }
    }
}
