use crate::ai::{
    AiModel, AiConfig, AiVendor, ModelTier, AiError,
    anthropic::AnthropicProvider,
    openai::OpenAiProvider,
    mistral::MistralProvider,
};
use std::sync::Arc;

/// Create an AI model instance based on the configuration
#[allow(unused)]
pub fn create_ai_model(
    config: AiConfig, 
    tier: ModelTier
) -> Result<Arc<dyn AiModel>, AiError> {
    // Create the appropriate model based on the configuration
    match config.vendor {
        AiVendor::Anthropic => {
            let model = AnthropicProvider::new(config, tier)?;
            Ok(Arc::new(model))
        },
        AiVendor::OpenAi => {
            let model = OpenAiProvider::new(config, tier)?;
            Ok(Arc::new(model))
        },
        AiVendor::Mistral => {
            let model = MistralProvider::new(config, tier)?;
            Ok(Arc::new(model))
        },
    }
}