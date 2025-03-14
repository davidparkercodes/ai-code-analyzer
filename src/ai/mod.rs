pub mod config;
pub mod anthropic;
pub mod openai;
pub mod mistral;
pub mod factory;
pub mod prompts;

pub use config::AiConfig;

use async_trait::async_trait;
use thiserror::Error;

/// Represents the tier level of the AI model to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    /// Low-tier model: faster, cheaper, less capable
    Low,
    /// Medium-tier model: balanced performance and capabilities
    Medium,
    /// High-tier model: most capable but slower and more expensive
    High,
}

impl Default for ModelTier {
    fn default() -> Self {
        ModelTier::Medium
    }
}

impl std::str::FromStr for ModelTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(ModelTier::Low),
            "medium" => Ok(ModelTier::Medium),
            "high" => Ok(ModelTier::High),
            _ => Err(format!("Invalid model tier: {}", s)),
        }
    }
}

/// AI vendor types supported by the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiVendor {
    Anthropic,
    OpenAi,
    Mistral,
}

impl Default for AiVendor {
    fn default() -> Self {
        AiVendor::Anthropic
    }
}

impl std::str::FromStr for AiVendor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "anthropic" => Ok(AiVendor::Anthropic),
            "openai" => Ok(AiVendor::OpenAi),
            "mistral" => Ok(AiVendor::Mistral),
            _ => Err(format!("Invalid AI vendor: {}", s)),
        }
    }
}

#[derive(Debug, Error)]
pub enum AiError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Trait defining the common interface for all AI models
#[async_trait]
pub trait AiModel: Send + Sync {
    /// Returns the name of this vendor
    #[allow(unused)]
    fn vendor_name(&self) -> &'static str;
    
    /// Returns the currently active model name
    #[allow(unused)]
    fn model_name(&self) -> String;
    
    /// Generate a text response from the AI model
    async fn generate_response(&self, prompt: &str) -> Result<String, AiError>;
    
    /// Generate code from the AI model
    #[allow(unused)]
    async fn generate_code(&self, prompt: &str, language: Option<&str>) -> Result<String, AiError>;
    
    /// Analyze code with the AI model
    #[allow(unused)]
    async fn analyze_code(&self, code: &str, prompt: Option<&str>) -> Result<String, AiError>;
}