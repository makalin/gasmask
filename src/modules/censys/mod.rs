use crate::config::Config;
use crate::modules::{Module, ModuleResult};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub struct CensysModule {
    client: Client,
}

impl CensysModule {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn search_hosts(
        &self,
        api_id: &str,
        api_secret: &str,
        query: &str,
        limit: u32,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        let auth = format!("{}:{}", api_id, api_secret);
        let auth_b64 = BASE64.encode(auth.as_bytes());

        let url = format!(
            "https://search.censys.io/api/v2/hosts/search?q={}&per_page={}",
            urlencoding::encode(query),
            limit
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Basic {}", auth_b64))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }

    async fn get_host_details(
        &self,
        api_id: &str,
        api_secret: &str,
        ip: &str,
    ) -> Result<serde_json::Value, Box<dyn Error>> {
        let auth = format!("{}:{}", api_id, api_secret);
        let auth_b64 = BASE64.encode(auth.as_bytes());

        let url = format!("https://search.censys.io/api/v2/hosts/{}", ip);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Basic {}", auth_b64))
            .send()
            .await?;

        let data = response.json::<serde_json::Value>().await?;
        Ok(data)
    }
}

#[async_trait]
impl Module for CensysModule {
    fn name(&self) -> &'static str {
        "censys"
    }

    fn description(&self) -> &'static str {
        "Censys information gathering module"
    }

    async fn run(&self, domain: &str, config: &Config) -> Result<ModuleResult, Box<dyn Error>> {
        let api_id = config.censys_api_id.as_ref().ok_or("Censys API ID not provided")?;
        let api_secret = config.censys_api_secret.as_ref().ok_or("Censys API Secret not provided")?;
        
        let mut data = Vec::new();
        let mut metadata = json!({});

        // Search for hosts
        let search_results = self.search_hosts(api_id, api_secret, &format!("names:{}", domain), config.limit).await?;
        
        if let Some(results) = search_results.get("result").and_then(|v| v.as_array()) {
            for result in results {
                if let Some(ip) = result.get("ip").and_then(|v| v.as_str()) {
                    data.push(format!("IP: {}", ip));

                    // Get detailed host information
                    if let Ok(host_info) = self.get_host_details(api_id, api_secret, ip).await {
                        if let Some(names) = host_info.get("names").and_then(|v| v.as_array()) {
                            for name in names {
                                if let Some(hostname) = name.as_str() {
                                    data.push(format!("Hostname: {}", hostname));
                                }
                            }
                        }

                        if let Some(ports) = host_info.get("ports").and_then(|v| v.as_array()) {
                            for port in ports {
                                if let Some(port_num) = port.as_u64() {
                                    data.push(format!("Open Port: {}", port_num));
                                }
                            }
                        }

                        if let Some(autonomous_system) = host_info.get("autonomous_system") {
                            if let Some(asn) = autonomous_system.get("asn").and_then(|v| v.as_u64()) {
                                data.push(format!("ASN: {}", asn));
                            }
                            if let Some(org) = autonomous_system.get("name").and_then(|v| v.as_str()) {
                                data.push(format!("Organization: {}", org));
                            }
                        }

                        metadata["host_info"] = host_info;
                    }
                }
            }
        }

        Ok(ModuleResult {
            source: "Censys".to_string(),
            data,
            metadata: Some(metadata),
        })
    }
} 