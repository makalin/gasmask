use crate::config::Config;
use crate::modules::{Module, ModuleResult};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub struct SpyseModule {
    client: Client,
}

impl SpyseModule {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn search_domain(
        &self,
        api_key: &str,
        domain: &str,
        limit: u32,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        let url = format!(
            "https://api.spyse.com/v4/data/domain/search?q={}&limit={}",
            urlencoding::encode(domain),
            limit
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    async fn get_domain_details(
        &self,
        api_key: &str,
        domain: &str,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        let url = format!(
            "https://api.spyse.com/v4/data/domain/{}",
            urlencoding::encode(domain)
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    async fn get_dns_records(
        &self,
        api_key: &str,
        domain: &str,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        let url = format!(
            "https://api.spyse.com/v4/data/domain/{}/dns",
            urlencoding::encode(domain)
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }
}

#[async_trait]
impl Module for SpyseModule {
    fn name(&self) -> &'static str {
        "spyse"
    }

    fn description(&self) -> &'static str {
        "Spyse information gathering module"
    }

    async fn run(&self, domain: &str, config: &Config) -> Result<ModuleResult, Box<dyn Error>> {
        let api_key = config.spyse_api_key.as_ref().ok_or("Spyse API key not provided")?;
        let mut data = Vec::new();
        let mut metadata = json!({});

        // Search for domain
        let search_results = self.search_domain(api_key, domain, config.limit).await?;
        if let Some(results) = search_results.get("data").and_then(|v| v.as_array()) {
            for result in results {
                if let Some(name) = result.get("name").and_then(|v| v.as_str()) {
                    data.push(format!("Domain: {}", name));
                }
            }
        }

        // Get domain details
        if let Ok(details) = self.get_domain_details(api_key, domain).await {
            if let Some(created_date) = details.get("created_date").and_then(|v| v.as_str()) {
                data.push(format!("Created Date: {}", created_date));
            }
            if let Some(expiry_date) = details.get("expiry_date").and_then(|v| v.as_str()) {
                data.push(format!("Expiry Date: {}", expiry_date));
            }
            if let Some(registrar) = details.get("registrar").and_then(|v| v.as_str()) {
                data.push(format!("Registrar: {}", registrar));
            }
            metadata["domain_details"] = details;
        }

        // Get DNS records
        if let Ok(dns_records) = self.get_dns_records(api_key, domain).await {
            if let Some(records) = dns_records.get("data").and_then(|v| v.as_array()) {
                for record in records {
                    if let Some(record_type) = record.get("type").and_then(|v| v.as_str()) {
                        if let Some(value) = record.get("value").and_then(|v| v.as_str()) {
                            data.push(format!("{} Record: {}", record_type, value));
                        }
                    }
                }
            }
            metadata["dns_records"] = dns_records;
        }

        Ok(ModuleResult {
            source: "Spyse".to_string(),
            data,
            metadata: Some(metadata),
        })
    }
} 