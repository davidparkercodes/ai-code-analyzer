pub mod config;
pub mod anthropic;
pub mod openai;
pub mod mistral;
pub mod factory;

pub use config::AiConfig;
pub use factory::create_ai_provider;

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

/// AI provider types supported by the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProvider {
    Anthropic,
    OpenAi,
    Mistral,
}

impl Default for AiProvider {
    fn default() -> Self {
        AiProvider::Anthropic
    }
}

impl std::str::FromStr for AiProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "anthropic" => Ok(AiProvider::Anthropic),
            "openai" => Ok(AiProvider::OpenAi),
            "mistral" => Ok(AiProvider::Mistral),
            _ => Err(format!("Invalid AI provider: {}", s)),
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
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Trait defining the common interface for all AI model providers
#[async_trait]
pub trait AiModelProvider: Send + Sync {
    /// Returns the name of this provider
    fn provider_name(&self) -> &'static str;
    
    /// Returns the currently active model name
    fn model_name(&self) -> String;
    
    /// Generate a text response from the AI model
    async fn generate_response(&self, prompt: &str) -> Result<String, AiError>;
    
    /// Generate code from the AI model
    async fn generate_code(&self, prompt: &str, language: Option<&str>) -> Result<String, AiError>;
    
    /// Analyze code with the AI model
    async fn analyze_code(&self, code: &str, prompt: Option<&str>) -> Result<String, AiError>;
}