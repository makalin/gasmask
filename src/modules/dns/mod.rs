use crate::config::Config;
use crate::modules::{Module, ModuleResult};
use async_trait::async_trait;
use std::error::Error;
use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;

pub struct DnsModule {
    resolver: Resolver,
}

impl DnsModule {
    pub fn new() -> Self {
        let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())
            .expect("Failed to create DNS resolver");
        Self { resolver }
    }

    async fn lookup_a(&self, domain: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let response = self.resolver.lookup_ip(domain).await?;
        Ok(response.iter().map(|ip| ip.to_string()).collect())
    }

    async fn lookup_mx(&self, domain: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let response = self.resolver.mx_lookup(domain).await?;
        Ok(response.iter().map(|mx| mx.exchange().to_string()).collect())
    }

    async fn lookup_ns(&self, domain: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let response = self.resolver.ns_lookup(domain).await?;
        Ok(response.iter().map(|ns| ns.to_string()).collect())
    }

    async fn lookup_txt(&self, domain: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let response = self.resolver.txt_lookup(domain).await?;
        Ok(response.iter().flat_map(|txt| txt.iter().map(|s| s.to_string())).collect())
    }
}

#[async_trait]
impl Module for DnsModule {
    fn name(&self) -> &'static str {
        "dns"
    }

    fn description(&self) -> &'static str {
        "DNS information gathering module"
    }

    async fn run(&self, domain: &str, _config: &Config) -> Result<ModuleResult, Box<dyn Error>> {
        let mut data = Vec::new();
        let mut metadata = serde_json::json!({});

        // Perform various DNS lookups
        if let Ok(ips) = self.lookup_a(domain).await {
            data.extend(ips);
            metadata["a_records"] = serde_json::json!(ips);
        }

        if let Ok(mx) = self.lookup_mx(domain).await {
            metadata["mx_records"] = serde_json::json!(mx);
        }

        if let Ok(ns) = self.lookup_ns(domain).await {
            metadata["ns_records"] = serde_json::json!(ns);
        }

        if let Ok(txt) = self.lookup_txt(domain).await {
            metadata["txt_records"] = serde_json::json!(txt);
        }

        Ok(ModuleResult {
            source: "DNS".to_string(),
            data,
            metadata: Some(metadata),
        })
    }
} 