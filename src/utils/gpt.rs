use serde::{Serialize, Deserialize};

use crate::secrets::GPT_KEY;

#[derive(Debug, Serialize, Deserialize)]
struct APIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletion {
    model: String,
    messages: Vec<APIMessage>,
}

pub async fn chat_api_request(user_chat: String) -> Result<String, String> {
    let url = "https://api.openai.com/v1/chat/completions";
    // println!("url: {}", url);
    let client = reqwest::Client::new();
    let messages = vec![APIMessage {
        role: "user".to_string(),
        content: user_chat,
    }];
    let chat_completion = ChatCompletion {
        model: "gpt-3.5-turbo".to_string(),
        messages,
    };
    // println!("client {:?}", client);
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", GPT_KEY))
        .json(&chat_completion)
        .send()
        .await
        .map_err(|e| format!("Error sending request: {}", e))?;

    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Error parsing response: {}", e))?;

    if let Some(content) = result
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
    {
        Ok(content.to_string())
    } else {
        Err("Message content not found in response".to_string())
    }
}
