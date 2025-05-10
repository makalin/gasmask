use crate::config::Config;
use crate::modules::{Module, ModuleResult};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::net::IpAddr;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

pub struct VhostsModule {
    client: Client,
}

impl VhostsModule {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn check_vhost(&self, ip: &str, hostname: &str) -> Result<bool, Box<dyn Error>> {
        let url = format!("http://{}", ip);
        let response = self.client
            .get(&url)
            .header("Host", hostname)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn resolve_ip(&self, domain: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let ips = tokio::net::lookup_host(format!("{}:80", domain)).await?;
        Ok(ips
            .into_iter()
            .filter_map(|addr| {
                if let IpAddr::V4(ipv4) = addr.ip() {
                    Some(ipv4.to_string())
                } else {
                    None
                }
            })
            .collect())
    }

    fn get_common_subdomains(&self) -> Vec<String> {
        vec![
            "www", "mail", "remote", "blog", "webmail", "server",
            "ns1", "ns2", "smtp", "secure", "vpn", "m", "shop",
            "ftp", "mail2", "test", "portal", "ns", "ww1", "host",
            "support", "dev", "web", "bbs", "ww42", "mx", "email",
            "cloud", "1", "mail1", "2", "forum", "owa", "www2",
            "gw", "admin", "store", "mx1", "cdn", "api", "exchange",
            "app", "gov", "2tty", "vps", "govyty", "hgfgdf", "we",
            "media", "ssl", "secure", "vpn", "m", "shop", "ftp",
            "mail2", "test", "portal", "ns", "ww1", "host", "support",
            "dev", "web", "bbs", "ww42", "mx", "email", "cloud",
            "1", "mail1", "2", "forum", "owa", "www2", "gw", "admin",
            "store", "mx1", "cdn", "api", "exchange", "app", "gov",
            "2tty", "vps", "govyty", "hgfgdf", "we", "media", "ssl",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

#[async_trait]
impl Module for VhostsModule {
    fn name(&self) -> &'static str {
        "vhosts"
    }

    fn description(&self) -> &'static str {
        "Virtual host detection module"
    }

    async fn run(&self, domain: &str, config: &Config) -> Result<ModuleResult, Box<dyn Error>> {
        let mut data = Vec::new();
        let mut metadata = json!({});
        let mut found_vhosts = Vec::new();

        // Resolve IP addresses
        let ips = self.resolve_ip(domain).await?;
        if ips.is_empty() {
            return Ok(ModuleResult {
                source: "VHosts".to_string(),
                data: vec!["No IP addresses found for domain".to_string()],
                metadata: None,
            });
        }

        // Check each IP address
        for ip in ips {
            data.push(format!("Checking IP: {}", ip));
            
            // Check common subdomains
            for subdomain in self.get_common_subdomains() {
                let hostname = format!("{}.{}", subdomain, domain);
                
                if self.check_vhost(&ip, &hostname).await? {
                    data.push(format!("Found virtual host: {}", hostname));
                    found_vhosts.push(hostname);
                }
                
                // Rate limiting
                sleep(Duration::from_millis(100)).await;
            }
        }

        metadata["found_vhosts"] = json!(found_vhosts);
        metadata["checked_ips"] = json!(ips);

        Ok(ModuleResult {
            source: "VHosts".to_string(),
            data,
            metadata: Some(metadata),
        })
    }
} 