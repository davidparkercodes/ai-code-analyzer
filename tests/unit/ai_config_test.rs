use std::env;
use ai_code_analyzer::ai::{AiConfig, AiVendor, ModelTier};

#[test]
fn test_default_config() {
    let config = AiConfig::default();
    
    assert_eq!(config.vendor, AiVendor::Anthropic);
    
    assert_eq!(config.get_model_name(AiVendor::Anthropic, ModelTier::Low), "claude-3-haiku-20240307");
    assert_eq!(config.get_model_name(AiVendor::Anthropic, ModelTier::Medium), "claude-3-sonnet-20240229");
    assert_eq!(config.get_model_name(AiVendor::Anthropic, ModelTier::High), "claude-3-opus-20240229");
    
    assert_eq!(config.get_model_name(AiVendor::OpenAi, ModelTier::Low), "gpt-3.5-turbo");
    assert_eq!(config.get_model_name(AiVendor::OpenAi, ModelTier::Medium), "gpt-4");
    assert_eq!(config.get_model_name(AiVendor::OpenAi, ModelTier::High), "gpt-4-turbo");
    
    assert_eq!(config.get_model_name(AiVendor::Mistral, ModelTier::Low), "mistral-tiny");
    assert_eq!(config.get_model_name(AiVendor::Mistral, ModelTier::Medium), "mistral-small");
    assert_eq!(config.get_model_name(AiVendor::Mistral, ModelTier::High), "mistral-large");
}

#[test]
fn test_model_tier_from_str() {
    assert_eq!("low".parse::<ModelTier>().unwrap(), ModelTier::Low);
    assert_eq!("medium".parse::<ModelTier>().unwrap(), ModelTier::Medium);
    assert_eq!("high".parse::<ModelTier>().unwrap(), ModelTier::High);
    
    assert_eq!("LOW".parse::<ModelTier>().unwrap(), ModelTier::Low);
    assert_eq!("MEDIUM".parse::<ModelTier>().unwrap(), ModelTier::Medium);
    assert_eq!("HIGH".parse::<ModelTier>().unwrap(), ModelTier::High);
    
    assert!("invalid".parse::<ModelTier>().is_err());
}

#[test]
fn test_vendor_from_str() {
    assert_eq!("anthropic".parse::<AiVendor>().unwrap(), AiVendor::Anthropic);
    assert_eq!("openai".parse::<AiVendor>().unwrap(), AiVendor::OpenAi);
    assert_eq!("mistral".parse::<AiVendor>().unwrap(), AiVendor::Mistral);
    
    assert_eq!("ANTHROPIC".parse::<AiVendor>().unwrap(), AiVendor::Anthropic);
    assert_eq!("OPENAI".parse::<AiVendor>().unwrap(), AiVendor::OpenAi);
    assert_eq!("MISTRAL".parse::<AiVendor>().unwrap(), AiVendor::Mistral);
    
    assert!("invalid".parse::<AiVendor>().is_err());
}

#[test]
fn test_api_keys() {
    let mut config = AiConfig::default();
    
    assert!(config.anthropic_api_key.is_none());
    assert!(config.openai_api_key.is_none());
    assert!(config.mistral_api_key.is_none());
    
    config.anthropic_api_key = Some("test-anthropic-key".to_string());
    config.openai_api_key = Some("test-openai-key".to_string());
    config.mistral_api_key = Some("test-mistral-key".to_string());
    
    assert_eq!(config.get_api_key(AiVendor::Anthropic).unwrap(), "test-anthropic-key");
    assert_eq!(config.get_api_key(AiVendor::OpenAi).unwrap(), "test-openai-key");
    assert_eq!(config.get_api_key(AiVendor::Mistral).unwrap(), "test-mistral-key");
}

#[test]
fn test_env_config() {
    
    let original_provider = env::var("AI_PROVIDER").ok();
    let original_anthropic_key = env::var("ANTHROPIC_API_KEY").ok();
    let original_openai_key = env::var("OPENAI_API_KEY").ok();
    let original_mistral_key = env::var("MISTRAL_API_KEY").ok();
    
    unsafe {
        env::set_var("AI_PROVIDER", "openai");
        env::set_var("OPENAI_API_KEY", "test-openai-key");
        env::set_var("OPENAI_HIGH_MODEL", "custom-gpt-model");
    }
    
    let config = AiConfig::from_env().unwrap();
    assert_eq!(config.vendor, AiVendor::OpenAi);
    assert_eq!(config.openai_api_key.as_ref().unwrap(), "test-openai-key");
    assert_eq!(config.get_model_name(AiVendor::OpenAi, ModelTier::High), "custom-gpt-model");
    
    unsafe {
        env::remove_var("AI_PROVIDER");
        env::remove_var("OPENAI_API_KEY");
        env::remove_var("OPENAI_HIGH_MODEL");
    
        if let Some(val) = original_provider {
            env::set_var("AI_PROVIDER", val);
        }
        if let Some(val) = original_anthropic_key {
            env::set_var("ANTHROPIC_API_KEY", val);
        }
        if let Some(val) = original_openai_key {
            env::set_var("OPENAI_API_KEY", val);
        }
        if let Some(val) = original_mistral_key {
            env::set_var("MISTRAL_API_KEY", val);
        }
    }
}
