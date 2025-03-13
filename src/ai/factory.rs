use crate::ai::{
    AiModelProvider, AiConfig, AiProvider, ModelTier,
    anthropic::AnthropicProvider,
    openai::OpenAiProvider,
    mistral::MistralProvider,
};
use std::sync::Arc;

/// Create an AI provider based on the configuration
pub fn create_ai_provider(
    config: AiConfig, 
    tier: ModelTier
) -> Result<Arc<dyn AiModelProvider>, AiError> {
    // Create the appropriate provider based on the configuration
    match config.provider {
        AiProvider::Anthropic => {
            let provider = AnthropicProvider::new(config, tier)?;
            Ok(Arc::new(provider))
        },
        AiProvider::OpenAi => {
            let provider = OpenAiProvider::new(config, tier)?;
            Ok(Arc::new(provider))
        },
        AiProvider::Mistral => {
            let provider = MistralProvider::new(config, tier)?;
            Ok(Arc::new(provider))
        },
    }
}