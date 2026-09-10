use super::{LlmClient, PromptRequest};
use reqwest::Client;
use serde_json::{Value, json};
use tokio::sync::mpsc;

pub struct GeminiClient {
    pub client: Client,
    pub api_key: String,
    pub model: String, // e.g., "gemini-1.5-pro"
}

impl LlmClient for GeminiClient {
    async fn stream_response(
        &self,
        req: PromptRequest,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<(), String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            self.model, self.api_key
        );

        let payload = json!({
            "system_instruction": {
                "parts": [{ "text": req.system_prompt }]
            },
            "contents": [{
                "role": "user",
                "parts": [{ "text": req.user_prompt }]
            }]
        });

        let mut res = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        while let Some(chunk) = res.chunk().await.map_err(|e| e.to_string())? {
            let chunk_str = String::from_utf8_lossy(&chunk);
            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if let Ok(parsed) = serde_json::from_str::<Value>(data) {
                        if let Some(text) =
                            parsed["candidates"][0]["content"]["parts"][0]["text"].as_str()
                        {
                            let _ = tx.send(text.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
