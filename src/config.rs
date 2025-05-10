use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub shodan_api_key: Option<String>,
    pub spyse_api_key: Option<String>,
    pub censys_api_id: Option<String>,
    pub censys_api_secret: Option<String>,
    pub proxy: Option<String>,
    pub dns_server: Option<String>,
    pub limit: u32,
    pub debug: bool,
    pub verbose: bool,
}

impl Config {
    pub fn new(args: &crate::Args) -> Result<Self> {
        let mut config = Self {
            shodan_api_key: args.shodan_key.clone(),
            spyse_api_key: args.spyse_key.clone(),
            censys_api_id: args.censys_api_id.clone(),
            censys_api_secret: args.censys_api_secret.clone(),
            proxy: args.proxy.clone(),
            dns_server: args.server.clone(),
            limit: args.limit,
            debug: args.debug,
            verbose: args.verbose,
        };

        // Try to load API keys from file if not provided in args
        config.load_api_keys()?;

        Ok(config)
    }

    fn load_api_keys(&mut self) -> Result<()> {
        let api_keys_path = Path::new("api_keys.txt");
        if api_keys_path.exists() {
            let contents = fs::read_to_string(api_keys_path)?;
            for line in contents.lines() {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    match parts[0].trim() {
                        "SHODAN_API_KEY" => {
                            if self.shodan_api_key.is_none() {
                                self.shodan_api_key = Some(parts[1].trim().to_string());
                            }
                        }
                        "SPYSE_API_KEY" => {
                            if self.spyse_api_key.is_none() {
                                self.spyse_api_key = Some(parts[1].trim().to_string());
                            }
                        }
                        "CENSYS_API_ID" => {
                            if self.censys_api_id.is_none() {
                                self.censys_api_id = Some(parts[1].trim().to_string());
                            }
                        }
                        "CENSYS_API_SECRET" => {
                            if self.censys_api_secret.is_none() {
                                self.censys_api_secret = Some(parts[1].trim().to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_api_keys(&self) -> Result<()> {
        let mut contents = String::new();
        
        if let Some(key) = &self.shodan_api_key {
            contents.push_str(&format!("SHODAN_API_KEY={}\n", key));
        }
        if let Some(key) = &self.spyse_api_key {
            contents.push_str(&format!("SPYSE_API_KEY={}\n", key));
        }
        if let Some(id) = &self.censys_api_id {
            contents.push_str(&format!("CENSYS_API_ID={}\n", id));
        }
        if let Some(secret) = &self.censys_api_secret {
            contents.push_str(&format!("CENSYS_API_SECRET={}\n", secret));
        }

        fs::write("api_keys.txt", contents)?;
        Ok(())
    }
} 