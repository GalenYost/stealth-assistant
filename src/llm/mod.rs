pub mod claude;
pub mod gemini;
pub mod openai;

use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum ProviderKind {
    Gemini,
    OpenAI,
    DeepSeek,
    Claude,
}

#[derive(Debug, Clone)]
pub struct PromptRequest {
    pub system_prompt: String,
    pub user_prompt: String,
}

pub trait LlmClient: Send + Sync {
    fn stream_response(
        &self,
        req: PromptRequest,
        tx: mpsc::UnboundedSender<String>,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}
