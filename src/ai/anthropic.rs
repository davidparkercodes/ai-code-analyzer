use crate::ai::{AiModel, AiError, ModelTier, AiConfig, AiVendor};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Implementation of the Anthropic Claude AI model provider
pub struct AnthropicProvider {
    config: AiConfig,
    client: Client,
    model_tier: ModelTier,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AnthropicContent {
    Text { text: String },
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicResponseContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponseContent {
    text: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the given configuration and model tier
    pub fn new(config: AiConfig, model_tier: ModelTier) -> Result<Self, AiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default();
            
        // Validate that we have an API key
        let _api_key = config.get_api_key(AiVendor::Anthropic)?;
            
        Ok(Self {
            config,
            client,
            model_tier,
        })
    }
    
    /// Get the API endpoint for Anthropic models
    fn api_endpoint(&self) -> &'static str {
        "https://api.anthropic.com/v1/messages"
    }
    
    /// Get the model name to use for the current tier
    fn get_model_name(&self) -> String {
        self.config.get_model_name(crate::ai::AiVendor::Anthropic, self.model_tier)
    }
    
    /// Get the API key
    fn get_api_key(&self) -> Result<String, AiError> {
        self.config.get_api_key(AiVendor::Anthropic)
    }
}

#[async_trait]
impl AiModel for AnthropicProvider {
    fn vendor_name(&self) -> &'static str {
        "Anthropic Claude"
    }
    
    fn model_name(&self) -> String {
        self.get_model_name()
    }
    
    async fn generate_response(&self, prompt: &str) -> Result<String, AiError> {
        let request = AnthropicRequest {
            model: self.get_model_name(),
            max_tokens: 4000,
            messages: vec![
                AnthropicMessage {
                    role: "user".to_string(),
                    content: vec![AnthropicContent::Text { 
                        text: prompt.to_string() 
                    }],
                },
            ],
        };
        
        let api_key = self.get_api_key()?;
        
        let response = self.client
            .post(self.api_endpoint())
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::Api(format!("Anthropic API error: {}", error_text)));
        }
        
        let response_data: AnthropicResponse = response.json().await?;
        
        // Extract the text from the first content item
        if let Some(content) = response_data.content.first() {
            Ok(content.text.clone())
        } else {
            Err(AiError::Api("No content in Anthropic response".to_string()))
        }
    }
    
    async fn generate_code(&self, prompt: &str, language: Option<&str>) -> Result<String, AiError> {
        let lang_str = language.unwrap_or("rust");
        
        let code_prompt = format!(
            "Generate {} code for the following task. Return ONLY code with no explanation. \
             The code should be complete, correct, and ready to use: {}", 
            lang_str, prompt
        );
        
        self.generate_response(&code_prompt).await
    }
    
    async fn analyze_code(&self, code: &str, prompt: Option<&str>) -> Result<String, AiError> {
        let analysis_prompt = match prompt {
            Some(p) => format!("Analyze this code:\n\n```\n{}\n```\n\n{}", code, p),
            None => format!("Analyze this code and provide insights on quality, possible improvements, and any issues:\n\n```\n{}\n```", code),
        };
        
        self.generate_response(&analysis_prompt).await
    }
}