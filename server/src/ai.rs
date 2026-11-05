//! Calls the LLM that powers group bots (F10). Talks to any OpenAI-compatible endpoint;
//! by default a local Ollama.

use std::time::Duration;

use aisdk::core::{DynamicModel, LanguageModelRequest};
use aisdk::providers::OpenAICompatible;

const DEFAULT_URL: &str = "http://localhost:11434/v1";
const DEFAULT_MODEL: &str = "llama3.2:1b";
const TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub enum AiError {
    Timeout,
    Request(aisdk::Error),
    Empty,
}

/// One bot reply. `system` is the bot's personality, `prompt` the conversation so far.
/// Config: `DEEPCHAT_AI_URL`, `DEEPCHAT_AI_MODEL`.
pub async fn complete(system: &str, prompt: &str) -> Result<String, AiError> {
    let url = std::env::var("DEEPCHAT_AI_URL").unwrap_or_else(|_| DEFAULT_URL.into());
    let model = std::env::var("DEEPCHAT_AI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.into());

    let provider = OpenAICompatible::<DynamicModel>::builder()
        .base_url(url)
        .api_key("unused") // Ollama ignores it
        .model_name(model)
        .build()
        .map_err(AiError::Request)?;

    let mut request = LanguageModelRequest::builder()
        .model(provider)
        .system(system)
        .prompt(prompt)
        .build();

    let response = tokio::time::timeout(TIMEOUT, request.generate_text())
        .await
        .map_err(|_| AiError::Timeout)?
        .map_err(AiError::Request)?;

    response.text().ok_or(AiError::Empty)
}

#[cfg(test)]
mod tests {
    // Needs Ollama running: cargo test -p server ai -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn ollama_replies() {
        let reply = super::complete("You talk like a pirate. Reply in one sentence.", "hi")
            .await
            .unwrap();
        println!("{reply}");
        assert!(!reply.is_empty());
    }
}
