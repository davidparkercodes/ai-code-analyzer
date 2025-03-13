use std::env;
use code_analyzer::ai::{AiConfig, AiProvider, ModelTier};

#[test]
fn test_default_config() {
    let config = AiConfig::default();
    
    // Check default values
    assert_eq!(config.provider, AiProvider::Anthropic);
    assert_eq!(config.default_tier, ModelTier::Medium);
    
    // Check default model names
    assert_eq!(config.get_model_name(AiProvider::Anthropic, ModelTier::Low), "claude-3-haiku-20240307");
    assert_eq!(config.get_model_name(AiProvider::Anthropic, ModelTier::Medium), "claude-3-sonnet-20240229");
    assert_eq!(config.get_model_name(AiProvider::Anthropic, ModelTier::High), "claude-3-opus-20240229");
    
    assert_eq!(config.get_model_name(AiProvider::OpenAi, ModelTier::Low), "gpt-3.5-turbo");
    assert_eq!(config.get_model_name(AiProvider::OpenAi, ModelTier::Medium), "gpt-4");
    assert_eq!(config.get_model_name(AiProvider::OpenAi, ModelTier::High), "gpt-4-turbo");
    
    assert_eq!(config.get_model_name(AiProvider::Mistral, ModelTier::Low), "mistral-tiny");
    assert_eq!(config.get_model_name(AiProvider::Mistral, ModelTier::Medium), "mistral-small");
    assert_eq!(config.get_model_name(AiProvider::Mistral, ModelTier::High), "mistral-large");
}

#[test]
fn test_model_tier_from_str() {
    assert_eq!("low".parse::<ModelTier>().unwrap(), ModelTier::Low);
    assert_eq!("medium".parse::<ModelTier>().unwrap(), ModelTier::Medium);
    assert_eq!("high".parse::<ModelTier>().unwrap(), ModelTier::High);
    
    // Case insensitive
    assert_eq!("LOW".parse::<ModelTier>().unwrap(), ModelTier::Low);
    assert_eq!("MEDIUM".parse::<ModelTier>().unwrap(), ModelTier::Medium);
    assert_eq!("HIGH".parse::<ModelTier>().unwrap(), ModelTier::High);
    
    // Invalid tier
    assert!("invalid".parse::<ModelTier>().is_err());
}

#[test]
fn test_provider_from_str() {
    assert_eq!("anthropic".parse::<AiProvider>().unwrap(), AiProvider::Anthropic);
    assert_eq!("openai".parse::<AiProvider>().unwrap(), AiProvider::OpenAi);
    assert_eq!("mistral".parse::<AiProvider>().unwrap(), AiProvider::Mistral);
    
    // Case insensitive
    assert_eq!("ANTHROPIC".parse::<AiProvider>().unwrap(), AiProvider::Anthropic);
    assert_eq!("OPENAI".parse::<AiProvider>().unwrap(), AiProvider::OpenAi);
    assert_eq!("MISTRAL".parse::<AiProvider>().unwrap(), AiProvider::Mistral);
    
    // Invalid provider
    assert!("invalid".parse::<AiProvider>().is_err());
}

#[test]
fn test_get_default_model_name() {
    let mut config = AiConfig::default();
    
    // Default is Anthropic Medium
    assert_eq!(config.get_default_model_name(), "claude-3-sonnet-20240229");
    
    // Change provider
    config.provider = AiProvider::OpenAi;
    assert_eq!(config.get_default_model_name(), "gpt-4");
    
    // Change tier
    config.default_tier = ModelTier::High;
    assert_eq!(config.get_default_model_name(), "gpt-4-turbo");
}

#[test]
fn test_env_config() {
    // This test depends on environment variables, so we need to be careful
    // about cleanup to avoid affecting other tests
    
    // Save original values
    let original_provider = env::var("AI_PROVIDER").ok();
    let original_tier = env::var("AI_TIER").ok();
    let original_api_key = env::var("AI_API_KEY").ok();
    
    // Set test values
    env::set_var("AI_PROVIDER", "openai");
    env::set_var("AI_TIER", "high");
    env::set_var("AI_API_KEY", "test-api-key");
    env::set_var("OPENAI_HIGH_MODEL", "custom-gpt-model");
    
    // Test config loading
    let config = AiConfig::from_env().unwrap();
    assert_eq!(config.provider, AiProvider::OpenAi);
    assert_eq!(config.default_tier, ModelTier::High);
    assert_eq!(config.api_key, "test-api-key");
    assert_eq!(config.get_default_model_name(), "custom-gpt-model");
    
    // Clean up
    env::remove_var("AI_PROVIDER");
    env::remove_var("AI_TIER");
    env::remove_var("AI_API_KEY");
    env::remove_var("OPENAI_HIGH_MODEL");
    
    // Restore original values if they existed
    if let Some(val) = original_provider {
        env::set_var("AI_PROVIDER", val);
    }
    if let Some(val) = original_tier {
        env::set_var("AI_TIER", val);
    }
    if let Some(val) = original_api_key {
        env::set_var("AI_API_KEY", val);
    }
}