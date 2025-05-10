use clap::Parser;
use colored::*;
use std::error::Error;
use std::time::Instant;

mod modules;
mod utils;
mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Domain to search
    #[arg(short, long)]
    domain: Option<String>,

    /// DNS server to use
    #[arg(short, long)]
    server: Option<String>,

    /// Use a proxy server
    #[arg(short, long)]
    proxy: Option<String>,

    /// Limit the number of search engine results
    #[arg(short, long, default_value_t = 100)]
    limit: u32,

    /// Information gathering mode
    #[arg(short, long)]
    info: Option<String>,

    /// Output base name
    #[arg(short, long)]
    output: Option<String>,

    /// Shodan API key
    #[arg(short, long)]
    shodan_key: Option<String>,

    /// Spyse API key
    #[arg(short, long)]
    spyse_key: Option<String>,

    /// Censys API ID
    #[arg(long)]
    censys_api_id: Option<String>,

    /// Censys API Secret
    #[arg(long)]
    censys_api_secret: Option<String>,

    /// Debug mode
    #[arg(short, long)]
    debug: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    // Print banner
    print_banner();

    // Initialize configuration
    let config = config::Config::new(&args)?;

    // Run the selected modules
    if let Some(domain) = args.domain {
        let start_time = Instant::now();
        let results = run_modules(&domain, &config, &args).await?;
        let duration = start_time.elapsed();

        // Print results
        print_results(&results, &args, duration)?;
    }

    Ok(())
}

fn print_banner() {
    println!(
        r#"
___________              .__                _________              
\__    ___/_  _  __ ____ |  |___  __ ____  /   _____/ ____   ____  
  |    |  \ \/ \/ // __ \|  |\  \/ // __ \ \_____  \_/ __ \_/ ___\ 
  |    |   \     /\  ___/|  |_\   /\  ___/ /        \  ___/\  \___ 
  |____|    \/\_/  \___  >____/\_/  \___  >_______  /\___  >\___  >
                       \/               \/        \/     \/     \/ 

GasMasK v. 2.0 - All in one Information gathering tool - OSINT
GasMasK is an open source tool licensed under GPLv3.
"#
    );
}

async fn run_modules(
    domain: &str,
    config: &config::Config,
    args: &Args,
) -> Result<Vec<modules::ModuleResult>, Box<dyn Error>> {
    let mut results = Vec::new();
    let modules_to_run = if let Some(info) = &args.info {
        info.split(',')
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
    } else {
        vec!["basic"]
    };

    for module_name in modules_to_run {
        if let Some(module) = modules::get_module_by_name(module_name) {
            println!("Running {} module...", module.name().green());
            match module.run(domain, config).await {
                Ok(result) => results.push(result),
                Err(e) => eprintln!("Error running {} module: {}", module.name().red(), e),
            }
        } else {
            eprintln!("Unknown module: {}", module_name.red());
        }
    }

    Ok(results)
}

fn print_results(
    results: &[modules::ModuleResult],
    args: &Args,
    duration: std::time::Duration,
) -> Result<(), Box<dyn Error>> {
    println!("\nResults:");
    println!("{}", "=".repeat(80));

    for result in results {
        println!("\nSource: {}", result.source.bold());
        println!("{}", "-".repeat(80));

        // Print data
        for item in &result.data {
            println!("{}", item);
        }

        // Print metadata if verbose
        if args.verbose {
            if let Some(metadata) = &result.metadata {
                println!("\nMetadata:");
                println!("{}", serde_json::to_string_pretty(metadata)?);
            }
        }
    }

    println!("\nScan completed in {}", utils::format_duration(duration).green());

    // Save results if output path is specified
    if let Some(output) = &args.output {
        let output_path = utils::sanitize_filename(output);
        let json = serde_json::to_string_pretty(&results)?;
        utils::save_results_to_file(&format!("{}.json", output_path), &json, "json")?;
        println!("Results saved to {}.json", output_path);
    }

    Ok(())
} 