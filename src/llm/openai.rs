use super::{LlmClient, PromptRequest};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::mpsc;

pub struct OpenAiClient {
    pub client: Client,
    pub api_key: String,
    pub base_url: String, // e.g., "https://api.openai.com/v1" or "https://api.deepseek.com"
    pub model: String,
}

impl LlmClient for OpenAiClient {
    async fn stream_response(
        &self,
        req: PromptRequest,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<(), String> {
        let payload = json!({
            "model": self.model,
            "stream": true,
            "messages": [
                {"role": "system", "content": req.system_prompt},
                {"role": "user", "content": req.user_prompt}
            ]
        });

        let mut res = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        while let Some(chunk) = res.chunk().await.map_err(|e| e.to_string())? {
            let chunk_str = String::from_utf8_lossy(&chunk);
            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        break;
                    }
                    if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                        if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                            let _ = tx.send(content.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
