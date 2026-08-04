//use rig::{completion::Prompt, client::{CompletionClient, ProviderClient},providers::openai};
use anyhow::Result;
use rig::{ client::CompletionClient, completion::Prompt, providers::{ openai}};
pub struct Agent_c {
    client:openai::Client,
}

impl Agent_c {
    pub fn new() -> Self {
        let client = openai::Client::builder().api_key(std::env::var("API").expect("failed to read api")).build().expect("failed to build client");
        
        Self { client }
    }
    pub async fn classify(&self, filename: &str) -> Result<String> {
        let prompt = format!("classify the file: {}", filename);
        let agent = self.client.agent("openai/gpt-4.1-mini").preamble("you are a file classification AI 
        calssify the given filename into one of the following categories:
        - Code
        - Image
        - Video
        - Audio
        - Document
        - Archive
        - Unknown").build();
        let response = agent.prompt(prompt).await?;

        Ok(response)
    }
}