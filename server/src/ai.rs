//! Calls the LLM that powers group bots (F10). Talks to any OpenAI-compatible endpoint;
//! by default a local Ollama.

use std::time::Duration;

use aisdk::core::{DynamicModel, LanguageModelRequest};
use aisdk::providers::OpenAICompatible;

const DEFAULT_URL: &str = "http://localhost:11434/v1";
const DEFAULT_MODEL: &str = "llama3.2:1b";
const TIMEOUT: Duration = Duration::from_secs(60);

/// One bot reply. `system` is the bot's personality, `prompt` the conversation so far.
/// Config: `DEEPCHAT_AI_URL`, `DEEPCHAT_AI_MODEL`, `DEEPCHAT_AI_KEY`.
pub async fn complete(system: &str, prompt: &str) -> Result<String, String> {
    let url = std::env::var("DEEPCHAT_AI_URL").unwrap_or_else(|_| DEFAULT_URL.into());
    let model = std::env::var("DEEPCHAT_AI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.into());
    let key = std::env::var("DEEPCHAT_AI_KEY").unwrap_or_else(|_| "unused".into()); // Ollama ignores it

    let provider = OpenAICompatible::<DynamicModel>::builder()
        .base_url(url)
        .api_key(key)
        .model_name(model)
        .build()
        .map_err(|e| e.to_string())?;

    let mut request = LanguageModelRequest::builder()
        .model(provider)
        .system(system)
        .prompt(prompt)
        .build();

    let response = tokio::time::timeout(TIMEOUT, request.generate_text())
        .await
        .map_err(|_| "timeout".to_string())?
        .map_err(|e| e.to_string())?;

    response.text().ok_or("empty reply".to_string())
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
