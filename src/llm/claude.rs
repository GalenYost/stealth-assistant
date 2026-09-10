use super::{LlmClient, PromptRequest};
use reqwest::Client;
use serde_json::{Value, json};
use tokio::sync::mpsc;

pub struct ClaudeClient {
    pub client: Client,
    pub api_key: String,
    pub model: String, // e.g., "claude-3-5-sonnet-20240620"
}

impl LlmClient for ClaudeClient {
    async fn stream_response(
        &self,
        req: PromptRequest,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<(), String> {
        let payload = json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": req.system_prompt,
            "stream": true,
            "messages": [
                {"role": "user", "content": req.user_prompt}
            ]
        });

        let mut res = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let mut is_content_block = false;

        while let Some(chunk) = res.chunk().await.map_err(|e| e.to_string())? {
            let chunk_str = String::from_utf8_lossy(&chunk);
            for line in chunk_str.lines() {
                if line.starts_with("event: ") {
                    is_content_block = line[7..] == *"content_block_delta";
                } else if line.starts_with("data: ") && is_content_block {
                    let data = &line[6..];
                    if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                        if let Some(text) = parsed["delta"]["text"].as_str() {
                            let _ = tx.send(text.to_string());
                        }
                    }
                    is_content_block = false;
                }
            }
        }
        Ok(())
    }
}
