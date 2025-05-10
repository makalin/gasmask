use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub mod dns;
pub mod whois;
pub mod search;
pub mod shodan;
pub mod censys;
pub mod spyse;
pub mod vhosts;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResult {
    pub source: String,
    pub data: Vec<String>,
    pub metadata: Option<serde_json::Value>,
}

#[async_trait]
pub trait Module: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn run(&self, domain: &str, config: &crate::config::Config) -> Result<ModuleResult, Box<dyn Error>>;
}

pub fn get_available_modules() -> Vec<&'static str> {
    vec![
        "basic",
        "nongoogle",
        "whois",
        "dns",
        "revdns",
        "vhosts",
        "google",
        "bing",
        "yahoo",
        "ask",
        "dogpile",
        "yandex",
        "linkedin",
        "twitter",
        "youtube",
        "reddit",
        "github",
        "instagram",
        "crt",
        "pgp",
        "netcraft",
        "virustotal",
        "dnsdump",
        "shodan",
        "censys",
        "spyse",
    ]
}

pub fn get_module_by_name(name: &str) -> Option<Box<dyn Module>> {
    match name {
        "whois" => Some(Box::new(whois::WhoisModule::new())),
        "dns" => Some(Box::new(dns::DnsModule::new())),
        "shodan" => Some(Box::new(shodan::ShodanModule::new())),
        "censys" => Some(Box::new(censys::CensysModule::new())),
        "spyse" => Some(Box::new(spyse::SpyseModule::new())),
        "vhosts" => Some(Box::new(vhosts::VhostsModule::new())),
        "search" => Some(Box::new(search::SearchModule::new())),
        "basic" => Some(Box::new(BasicModule::new())),
        "nongoogle" => Some(Box::new(NonGoogleModule::new())),
        _ => None,
    }
}

pub struct BasicModule {
    whois: whois::WhoisModule,
    dns: dns::DnsModule,
    vhosts: vhosts::VhostsModule,
}

impl BasicModule {
    pub fn new() -> Self {
        Self {
            whois: whois::WhoisModule::new(),
            dns: dns::DnsModule::new(),
            vhosts: vhosts::VhostsModule::new(),
        }
    }
}

#[async_trait]
impl Module for BasicModule {
    fn name(&self) -> &'static str {
        "basic"
    }

    fn description(&self) -> &'static str {
        "Basic information gathering module"
    }

    async fn run(&self, domain: &str, config: &crate::config::Config) -> Result<ModuleResult, Box<dyn Error>> {
        let mut data = Vec::new();
        let mut metadata = serde_json::json!({});

        // Run WHOIS
        if let Ok(result) = self.whois.run(domain, config).await {
            data.extend(result.data);
            if let Some(whois_metadata) = result.metadata {
                metadata["whois"] = whois_metadata;
            }
        }

        // Run DNS
        if let Ok(result) = self.dns.run(domain, config).await {
            data.extend(result.data);
            if let Some(dns_metadata) = result.metadata {
                metadata["dns"] = dns_metadata;
            }
        }

        // Run VHosts
        if let Ok(result) = self.vhosts.run(domain, config).await {
            data.extend(result.data);
            if let Some(vhosts_metadata) = result.metadata {
                metadata["vhosts"] = vhosts_metadata;
            }
        }

        Ok(ModuleResult {
            source: "Basic".to_string(),
            data,
            metadata: Some(metadata),
        })
    }
}

pub struct NonGoogleModule {
    basic: BasicModule,
    search: search::SearchModule,
}

impl NonGoogleModule {
    pub fn new() -> Self {
        Self {
            basic: BasicModule::new(),
            search: search::SearchModule::new(),
        }
    }
}

#[async_trait]
impl Module for NonGoogleModule {
    fn name(&self) -> &'static str {
        "nongoogle"
    }

    fn description(&self) -> &'static str {
        "Non-Google information gathering module"
    }

    async fn run(&self, domain: &str, config: &crate::config::Config) -> Result<ModuleResult, Box<dyn Error>> {
        let mut data = Vec::new();
        let mut metadata = serde_json::json!({});

        // Run basic modules
        if let Ok(result) = self.basic.run(domain, config).await {
            data.extend(result.data);
            if let Some(basic_metadata) = result.metadata {
                metadata["basic"] = basic_metadata;
            }
        }

        // Run search module
        if let Ok(result) = self.search.run(domain, config).await {
            data.extend(result.data);
            if let Some(search_metadata) = result.metadata {
                metadata["search"] = search_metadata;
            }
        }

        Ok(ModuleResult {
            source: "Non-Google".to_string(),
            data,
            metadata: Some(metadata),
        })
    }
} 