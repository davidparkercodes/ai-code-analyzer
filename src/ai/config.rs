use crate::ai::{AiVendor, ModelTier, AiError};
use dotenv::dotenv;
use std::env;

/// Configuration for AI services
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// The AI vendor to use (Anthropic, OpenAI, Mistral)
    pub vendor: AiVendor,
    
    /// API keys for each provider
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub mistral_api_key: Option<String>,
    
    pub anthropic_low_model: String,
    pub anthropic_medium_model: String,
    pub anthropic_high_model: String,
    
    pub openai_low_model: String,
    pub openai_medium_model: String,
    pub openai_high_model: String,
    
    pub mistral_low_model: String,
    pub mistral_medium_model: String,
    pub mistral_high_model: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            vendor: AiVendor::default(),
            
            anthropic_api_key: None,
            openai_api_key: None,
            mistral_api_key: None,
            
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
        dotenv().ok();
        
        let mut config = Self::default();
        
        if let Ok(vendor_str) = env::var("AI_PROVIDER") {
            config.vendor = vendor_str.parse().map_err(|e| AiError::Config(e))?;
        }
        
        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            config.anthropic_api_key = Some(key);
        }
        
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            config.openai_api_key = Some(key);
        }
        
        if let Ok(key) = env::var("MISTRAL_API_KEY") {
            config.mistral_api_key = Some(key);
        }
        
        Self::set_model_if_exists(&mut config.anthropic_low_model, "ANTHROPIC_LOW_MODEL");
        Self::set_model_if_exists(&mut config.anthropic_medium_model, "ANTHROPIC_MEDIUM_MODEL");
        Self::set_model_if_exists(&mut config.anthropic_high_model, "ANTHROPIC_HIGH_MODEL");
        
        Self::set_model_if_exists(&mut config.openai_low_model, "OPENAI_LOW_MODEL");
        Self::set_model_if_exists(&mut config.openai_medium_model, "OPENAI_MEDIUM_MODEL");
        Self::set_model_if_exists(&mut config.openai_high_model, "OPENAI_HIGH_MODEL");
        
        Self::set_model_if_exists(&mut config.mistral_low_model, "MISTRAL_LOW_MODEL");
        Self::set_model_if_exists(&mut config.mistral_medium_model, "MISTRAL_MEDIUM_MODEL");
        Self::set_model_if_exists(&mut config.mistral_high_model, "MISTRAL_HIGH_MODEL");
        
        match config.vendor {
            AiVendor::Anthropic if config.anthropic_api_key.is_none() => {
                return Err(AiError::Config("Missing ANTHROPIC_API_KEY for Anthropic vendor".to_string()));
            },
            AiVendor::OpenAi if config.openai_api_key.is_none() => {
                return Err(AiError::Config("Missing OPENAI_API_KEY for OpenAI vendor".to_string()));
            },
            AiVendor::Mistral if config.mistral_api_key.is_none() => {
                return Err(AiError::Config("Missing MISTRAL_API_KEY for Mistral vendor".to_string()));
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
    
    /// Get the model name for the specified vendor and tier
    pub fn get_model_name(&self, vendor: AiVendor, tier: ModelTier) -> String {
        match (vendor, tier) {
            (AiVendor::Anthropic, ModelTier::Low) => self.anthropic_low_model.clone(),
            (AiVendor::Anthropic, ModelTier::Medium) => self.anthropic_medium_model.clone(),
            (AiVendor::Anthropic, ModelTier::High) => self.anthropic_high_model.clone(),
            
            (AiVendor::OpenAi, ModelTier::Low) => self.openai_low_model.clone(),
            (AiVendor::OpenAi, ModelTier::Medium) => self.openai_medium_model.clone(),
            (AiVendor::OpenAi, ModelTier::High) => self.openai_high_model.clone(),
            
            (AiVendor::Mistral, ModelTier::Low) => self.mistral_low_model.clone(),
            (AiVendor::Mistral, ModelTier::Medium) => self.mistral_medium_model.clone(),
            (AiVendor::Mistral, ModelTier::High) => self.mistral_high_model.clone(),
        }
    }
    
    /// Get the API key for the specified vendor
    pub fn get_api_key(&self, vendor: AiVendor) -> Result<String, AiError> {
        match vendor {
            AiVendor::Anthropic => {
                self.anthropic_api_key.clone()
                    .ok_or_else(|| AiError::Config("Missing Anthropic API key".to_string()))
            },
            AiVendor::OpenAi => {
                self.openai_api_key.clone()
                    .ok_or_else(|| AiError::Config("Missing OpenAI API key".to_string()))
            },
            AiVendor::Mistral => {
                self.mistral_api_key.clone()
                    .ok_or_else(|| AiError::Config("Missing Mistral API key".to_string()))
            },
        }
    }
}
