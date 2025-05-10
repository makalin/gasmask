use crate::config::Config;
use crate::modules::{Module, ModuleResult};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

pub struct SearchModule {
    client: Client,
}

impl SearchModule {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        Self { client }
    }

    async fn search_google(&self, domain: &str, limit: u32) -> Result<Vec<String>, Box<dyn Error>> {
        let mut results = Vec::new();
        let mut start = 0;
        
        while results.len() < limit as usize {
            let url = format!(
                "https://www.google.com/search?q=site:{}&start={}",
                urlencoding::encode(domain),
                start
            );

            let response = self.client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .send()
                .await?;

            let text = response.text().await?;
            
            // Extract URLs from Google search results
            if let Some(links) = extract_google_links(&text) {
                results.extend(links);
            }

            start += 10;
            if start >= 100 { // Google typically limits to 100 results
                break;
            }

            // Rate limiting
            sleep(Duration::from_secs(2)).await;
        }

        Ok(results.into_iter().take(limit as usize).collect())
    }

    async fn search_bing(&self, domain: &str, limit: u32) -> Result<Vec<String>, Box<dyn Error>> {
        let mut results = Vec::new();
        let mut offset = 0;
        
        while results.len() < limit as usize {
            let url = format!(
                "https://www.bing.com/search?q=site:{}&first={}",
                urlencoding::encode(domain),
                offset
            );

            let response = self.client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .send()
                .await?;

            let text = response.text().await?;
            
            // Extract URLs from Bing search results
            if let Some(links) = extract_bing_links(&text) {
                results.extend(links);
            }

            offset += 10;
            if offset >= 100 { // Bing typically limits to 100 results
                break;
            }

            // Rate limiting
            sleep(Duration::from_secs(2)).await;
        }

        Ok(results.into_iter().take(limit as usize).collect())
    }

    async fn search_github(&self, domain: &str, limit: u32) -> Result<Vec<String>, Box<dyn Error>> {
        let mut results = Vec::new();
        let mut page = 1;
        
        while results.len() < limit as usize {
            let url = format!(
                "https://github.com/search?q={}&type=code&p={}",
                urlencoding::encode(domain),
                page
            );

            let response = self.client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .send()
                .await?;

            let text = response.text().await?;
            
            // Extract URLs from GitHub search results
            if let Some(links) = extract_github_links(&text) {
                results.extend(links);
            }

            page += 1;
            if page > 10 { // GitHub typically limits to 10 pages
                break;
            }

            // Rate limiting
            sleep(Duration::from_secs(2)).await;
        }

        Ok(results.into_iter().take(limit as usize).collect())
    }
}

fn extract_google_links(html: &str) -> Option<Vec<String>> {
    let mut links = Vec::new();
    let re = regex::Regex::new(r#"<a href="(https?://[^"]+)"[^>]*>"#).ok()?;
    
    for cap in re.captures_iter(html) {
        if let Some(url) = cap.get(1) {
            let url = url.as_str();
            if !url.contains("google.com") {
                links.push(url.to_string());
            }
        }
    }
    
    Some(links)
}

fn extract_bing_links(html: &str) -> Option<Vec<String>> {
    let mut links = Vec::new();
    let re = regex::Regex::new(r#"<a href="(https?://[^"]+)"[^>]*>"#).ok()?;
    
    for cap in re.captures_iter(html) {
        if let Some(url) = cap.get(1) {
            let url = url.as_str();
            if !url.contains("bing.com") {
                links.push(url.to_string());
            }
        }
    }
    
    Some(links)
}

fn extract_github_links(html: &str) -> Option<Vec<String>> {
    let mut links = Vec::new();
    let re = regex::Regex::new(r#"<a href="(/[^"]+)"[^>]*>"#).ok()?;
    
    for cap in re.captures_iter(html) {
        if let Some(path) = cap.get(1) {
            let path = path.as_str();
            if path.starts_with("/") && !path.contains("github.com") {
                links.push(format!("https://github.com{}", path));
            }
        }
    }
    
    Some(links)
}

#[async_trait]
impl Module for SearchModule {
    fn name(&self) -> &'static str {
        "search"
    }

    fn description(&self) -> &'static str {
        "Search engine information gathering module"
    }

    async fn run(&self, domain: &str, config: &Config) -> Result<ModuleResult, Box<dyn Error>> {
        let mut data = Vec::new();
        let mut metadata = json!({});
        let mut all_results = Vec::new();

        // Search Google
        if let Ok(results) = self.search_google(domain, config.limit).await {
            data.push(format!("Found {} results from Google", results.len()));
            all_results.extend(results);
            metadata["google_results"] = json!(results);
        }

        // Search Bing
        if let Ok(results) = self.search_bing(domain, config.limit).await {
            data.push(format!("Found {} results from Bing", results.len()));
            all_results.extend(results);
            metadata["bing_results"] = json!(results);
        }

        // Search GitHub
        if let Ok(results) = self.search_github(domain, config.limit).await {
            data.push(format!("Found {} results from GitHub", results.len()));
            all_results.extend(results);
            metadata["github_results"] = json!(results);
        }

        // Remove duplicates and sort
        all_results.sort();
        all_results.dedup();

        metadata["total_results"] = json!(all_results.len());
        metadata["all_results"] = json!(all_results);

        Ok(ModuleResult {
            source: "Search Engines".to_string(),
            data,
            metadata: Some(metadata),
        })
    }
} 