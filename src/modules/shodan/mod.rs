use crate::config::Config;
use crate::modules::{Module, ModuleResult};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;

pub struct ShodanModule {
    client: Client,
}

impl ShodanModule {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn search_host(&self, api_key: &str, query: &str) -> Result<Value, Box<dyn Error>> {
        let url = format!(
            "https://api.shodan.io/shodan/host/search?key={}&query={}",
            api_key, query
        );
        let response = self.client.get(&url).send().await?;
        let data = response.json::<Value>().await?;
        Ok(data)
    }

    async fn get_host_info(&self, api_key: &str, ip: &str) -> Result<Value, Box<dyn Error>> {
        let url = format!(
            "https://api.shodan.io/shodan/host/{}?key={}",
            ip, api_key
        );
        let response = self.client.get(&url).send().await?;
        let data = response.json::<Value>().await?;
        Ok(data)
    }

    async fn get_dns_resolve(&self, api_key: &str, domains: &[&str]) -> Result<Value, Box<dyn Error>> {
        let url = format!(
            "https://api.shodan.io/dns/resolve?key={}&hostnames={}",
            api_key,
            domains.join(",")
        );
        let response = self.client.get(&url).send().await?;
        let data = response.json::<Value>().await?;
        Ok(data)
    }

    async fn get_dns_reverse(&self, api_key: &str, ips: &[&str]) -> Result<Value, Box<dyn Error>> {
        let url = format!(
            "https://api.shodan.io/dns/reverse?key={}&ips={}",
            api_key,
            ips.join(",")
        );
        let response = self.client.get(&url).send().await?;
        let data = response.json::<Value>().await?;
        Ok(data)
    }
}

#[async_trait]
impl Module for ShodanModule {
    fn name(&self) -> &'static str {
        "shodan"
    }

    fn description(&self) -> &'static str {
        "Shodan information gathering module"
    }

    async fn run(&self, domain: &str, config: &Config) -> Result<ModuleResult, Box<dyn Error>> {
        let api_key = config.shodan_api_key.as_ref().ok_or("Shodan API key not provided")?;
        let mut data = Vec::new();
        let mut metadata = serde_json::json!({});

        // Search for the domain
        let search_results = self.search_host(api_key, &format!("hostname:{}", domain)).await?;
        if let Some(matches) = search_results.get("matches") {
            if let Some(matches_array) = matches.as_array() {
                for match_data in matches_array {
                    if let Some(ip) = match_data.get("ip_str").and_then(|v| v.as_str()) {
                        data.push(format!("IP: {}", ip));
                        
                        // Get detailed host information
                        if let Ok(host_info) = self.get_host_info(api_key, ip).await {
                            if let Some(hostnames) = host_info.get("hostnames").and_then(|v| v.as_array()) {
                                for hostname in hostnames {
                                    if let Some(name) = hostname.as_str() {
                                        data.push(format!("Hostname: {}", name));
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

                            if let Some(org) = host_info.get("org").and_then(|v| v.as_str()) {
                                data.push(format!("Organization: {}", org));
                            }

                            if let Some(os) = host_info.get("os").and_then(|v| v.as_str()) {
                                data.push(format!("Operating System: {}", os));
                            }

                            metadata["host_info"] = host_info;
                        }
                    }
                }
            }
        }

        // Get DNS resolution
        if let Ok(dns_data) = self.get_dns_resolve(api_key, &[domain]).await {
            metadata["dns_resolve"] = dns_data;
        }

        Ok(ModuleResult {
            source: "Shodan".to_string(),
            data,
            metadata: Some(metadata),
        })
    }
} 