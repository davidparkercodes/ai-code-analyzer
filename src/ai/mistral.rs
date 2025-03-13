use crate::ai::{AiModel, AiError, ModelTier, AiConfig, AiVendor};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Implementation of the Mistral AI model provider
pub struct MistralProvider {
    config: AiConfig,
    client: Client,
    model_tier: ModelTier,
}

#[derive(Debug, Serialize)]
struct MistralRequest {
    model: String,
    messages: Vec<MistralMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Serialize)]
struct MistralMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct MistralResponse {
    choices: Vec<MistralChoice>,
}

#[derive(Debug, Deserialize)]
struct MistralChoice {
    message: MistralResponseMessage,
}

#[derive(Debug, Deserialize)]
struct MistralResponseMessage {
    content: String,
}

impl MistralProvider {
    /// Create a new Mistral provider with the given configuration and model tier
    #[allow(unused)]
    pub fn new(config: AiConfig, model_tier: ModelTier) -> Result<Self, AiError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default();
            
        // Validate that we have an API key
        let _api_key = config.get_api_key(AiVendor::Mistral)?;
            
        Ok(Self {
            config,
            client,
            model_tier,
        })
    }
    
    /// Get the API endpoint for Mistral models
    fn api_endpoint(&self) -> &'static str {
        "https://api.mistral.ai/v1/chat/completions"
    }
    
    /// Get the model name to use for the current tier
    fn get_model_name(&self) -> String {
        self.config.get_model_name(crate::ai::AiVendor::Mistral, self.model_tier)
    }
    
    /// Get the API key
    fn get_api_key(&self) -> Result<String, AiError> {
        self.config.get_api_key(AiVendor::Mistral)
    }
}

#[async_trait]
impl AiModel for MistralProvider {
    fn vendor_name(&self) -> &'static str {
        "Mistral AI"
    }
    
    fn model_name(&self) -> String {
        self.get_model_name()
    }
    
    async fn generate_response(&self, prompt: &str) -> Result<String, AiError> {
        let request = MistralRequest {
            model: self.get_model_name(),
            messages: vec![
                MistralMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            max_tokens: Some(4000),
            temperature: Some(0.7),
        };
        
        let api_key = self.get_api_key()?;
        
        let response = self.client
            .post(self.api_endpoint())
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::Api(format!("Mistral API error: {}", error_text)));
        }
        
        let response_data: MistralResponse = response.json().await?;
        
        // Extract the content from the first choice
        if let Some(choice) = response_data.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(AiError::Api("No choices in Mistral response".to_string()))
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