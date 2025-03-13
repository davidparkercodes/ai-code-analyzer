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
    tier: Option<ModelTier>
) -> Arc<dyn AiModelProvider> {
    // Use the provided tier or fall back to the default from config
    let model_tier = tier.unwrap_or(config.default_tier);
    
    // Create the appropriate provider based on the configuration
    match config.provider {
        AiProvider::Anthropic => {
            Arc::new(AnthropicProvider::new(config, model_tier))
        },
        AiProvider::OpenAi => {
            Arc::new(OpenAiProvider::new(config, model_tier))
        },
        AiProvider::Mistral => {
            Arc::new(MistralProvider::new(config, model_tier))
        },
    }
}