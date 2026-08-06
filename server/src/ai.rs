//use rig::{completion::Prompt, client::{CompletionClient, ProviderClient},providers::openai};
use anyhow::Result;
use rig::{ client::CompletionClient, completion::Prompt, providers::{ openrouter}};
pub struct Agent_c {
    client:openrouter::Client,
} 


impl Agent_c {  
    pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").expect("failed to read api key");
        //let api_key_d ="sk-f014c9efae494d7eb0d24c6cfc383686".to_string();
        let client = openrouter::Client::builder().api_key(api_key).build().expect("failed to build client");
        
        Self { client }
    }
    pub async fn classify(&self, filename: &str) -> Result<String> {
        let prompt = format!("classify the file: {}", filename);
        let agent = self.client.agent("openrouter/free").preamble("you are a file classification AI 
        calssify the given filename into one of the following categories:
        - Code
        - Image
        - Video
        - Audio
        - Document
        - Archive
        - Unknown
        only return the category name as a single word").max_tokens(10).build();
        let response = agent.prompt(prompt).await?;

        Ok(response)
    }
}