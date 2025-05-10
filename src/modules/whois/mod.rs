use crate::config::Config;
use crate::modules::{Module, ModuleResult};
use async_trait::async_trait;
use std::error::Error;
use whois::WhoisLookup;

pub struct WhoisModule {
    client: WhoisLookup,
}

impl WhoisModule {
    pub fn new() -> Self {
        Self {
            client: WhoisLookup::new(),
        }
    }
}

#[async_trait]
impl Module for WhoisModule {
    fn name(&self) -> &'static str {
        "whois"
    }

    fn description(&self) -> &'static str {
        "WHOIS information gathering module"
    }

    async fn run(&self, domain: &str, _config: &Config) -> Result<ModuleResult, Box<dyn Error>> {
        let whois_data = self.client.lookup(domain)?;
        
        let mut data = Vec::new();
        let mut metadata = serde_json::json!({});

        // Parse WHOIS data into structured format
        for line in whois_data.lines() {
            if !line.trim().is_empty() {
                data.push(line.trim().to_string());
                
                // Try to parse key-value pairs
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim().to_lowercase().replace(' ', "_");
                    let value = value.trim();
                    if !value.is_empty() {
                        metadata[key] = serde_json::json!(value);
                    }
                }
            }
        }

        Ok(ModuleResult {
            source: "WHOIS".to_string(),
            data,
            metadata: Some(metadata),
        })
    }
} 