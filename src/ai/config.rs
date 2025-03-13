use crate::ai::{AiProvider, ModelTier, AiError};
use dotenv::dotenv;
use std::env;

/// Configuration for AI services
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// The AI provider to use (Anthropic, OpenAI, Mistral)
    pub provider: AiProvider,
    
    /// API keys for each provider
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub mistral_api_key: Option<String>,
    
    // Model names for each provider and tier
    // Anthropic
    pub anthropic_low_model: String,
    pub anthropic_medium_model: String,
    pub anthropic_high_model: String,
    
    // OpenAI
    pub openai_low_model: String,
    pub openai_medium_model: String,
    pub openai_high_model: String,
    
    // Mistral
    pub mistral_low_model: String,
    pub mistral_medium_model: String,
    pub mistral_high_model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::default(),
            
            // API keys
            anthropic_api_key: None,
            openai_api_key: None,
            mistral_api_key: None,
            
            // Default model names
            anthropic_low_model: "claude-3-haiku-20240307".to_string(),
            anthropic_medium_model: "claude-3-sonnet-20240229".to_string(),
            anthropic_high_model: "claude-3-opus-20240229".to_string(),
            
            openai_low_model: "gpt-3.5-turbo".to_string(),
            openai_medium_model: "gpt-4".to_string(),
            openai_high_model: "gpt-4-turbo".to_string(),
            
            mistral_low_model: "mistral-tiny".to_string(),
            mistral_medium_model: "mistral-small".to_string(),
            mistral_high_model: "mistral-large".to_string(),
        }
    }
}

impl AiConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, AiError> {
        // Load .env file if present
        dotenv().ok();
        
        // Start with defaults
        let mut config = Self::default();
        
        // Set provider
        if let Ok(provider_str) = env::var("AI_PROVIDER") {
            config.provider = provider_str.parse().map_err(|e| AiError::Config(e))?;
        }
        
        // Set API keys for each provider
        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            config.anthropic_api_key = Some(key);
        }
        
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            config.openai_api_key = Some(key);
        }
        
        if let Ok(key) = env::var("MISTRAL_API_KEY") {
            config.mistral_api_key = Some(key);
        }
        
        // Set model names (if provided)
        Self::set_model_if_exists(&mut config.anthropic_low_model, "ANTHROPIC_LOW_MODEL");
        Self::set_model_if_exists(&mut config.anthropic_medium_model, "ANTHROPIC_MEDIUM_MODEL");
        Self::set_model_if_exists(&mut config.anthropic_high_model, "ANTHROPIC_HIGH_MODEL");
        
        Self::set_model_if_exists(&mut config.openai_low_model, "OPENAI_LOW_MODEL");
        Self::set_model_if_exists(&mut config.openai_medium_model, "OPENAI_MEDIUM_MODEL");
        Self::set_model_if_exists(&mut config.openai_high_model, "OPENAI_HIGH_MODEL");
        
        Self::set_model_if_exists(&mut config.mistral_low_model, "MISTRAL_LOW_MODEL");
        Self::set_model_if_exists(&mut config.mistral_medium_model, "MISTRAL_MEDIUM_MODEL");
        Self::set_model_if_exists(&mut config.mistral_high_model, "MISTRAL_HIGH_MODEL");
        
        // Validate that we have an API key for the selected provider
        match config.provider {
            AiProvider::Anthropic if config.anthropic_api_key.is_none() => {
                return Err(AiError::Config("Missing ANTHROPIC_API_KEY for Anthropic provider".to_string()));
            },
            AiProvider::OpenAi if config.openai_api_key.is_none() => {
                return Err(AiError::Config("Missing OPENAI_API_KEY for OpenAI provider".to_string()));
            },
            AiProvider::Mistral if config.mistral_api_key.is_none() => {
                return Err(AiError::Config("Missing MISTRAL_API_KEY for Mistral provider".to_string()));
            },
            _ => {}
        }
        
        Ok(config)
    }
    
    /// Helper to set model name from env if it exists
    fn set_model_if_exists(target: &mut String, env_var: &str) {
        if let Ok(value) = env::var(env_var) {
            *target = value;
        }
    }
    
    /// Get the model name for the specified provider and tier
    pub fn get_model_name(&self, provider: AiProvider, tier: ModelTier) -> String {
        match (provider, tier) {
            (AiProvider::Anthropic, ModelTier::Low) => self.anthropic_low_model.clone(),
            (AiProvider::Anthropic, ModelTier::Medium) => self.anthropic_medium_model.clone(),
            (AiProvider::Anthropic, ModelTier::High) => self.anthropic_high_model.clone(),
            
            (AiProvider::OpenAi, ModelTier::Low) => self.openai_low_model.clone(),
            (AiProvider::OpenAi, ModelTier::Medium) => self.openai_medium_model.clone(),
            (AiProvider::OpenAi, ModelTier::High) => self.openai_high_model.clone(),
            
            (AiProvider::Mistral, ModelTier::Low) => self.mistral_low_model.clone(),
            (AiProvider::Mistral, ModelTier::Medium) => self.mistral_medium_model.clone(),
            (AiProvider::Mistral, ModelTier::High) => self.mistral_high_model.clone(),
        }
    }
    
    /// Get the API key for the specified provider
    pub fn get_api_key(&self, provider: AiProvider) -> Result<String, AiError> {
        match provider {
            AiProvider::Anthropic => {
                self.anthropic_api_key.clone()
                    .ok_or_else(|| AiError::Config("Missing Anthropic API key".to_string()))
            },
            AiProvider::OpenAi => {
                self.openai_api_key.clone()
                    .ok_or_else(|| AiError::Config("Missing OpenAI API key".to_string()))
            },
            AiProvider::Mistral => {
                self.mistral_api_key.clone()
                    .ok_or_else(|| AiError::Config("Missing Mistral API key".to_string()))
            },
        }
    }
}