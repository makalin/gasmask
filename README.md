# GasMask

A modern Rust implementation of an OSINT information gathering tool.

This is a fork of [twelvesec/gasmask](https://github.com/twelvesec/gasmask) with custom modifications and improvements.

## Features

* Modern Rust implementation
* Async/await for better performance
* Modular architecture
* JSON output support
* Colored terminal output
* Progress indicators
* Comprehensive error handling

## Information Gathering Modules

* DNS queries
* WHOIS lookup
* Shodan integration
* Censys integration
* Spyse integration
* Virtual host detection
* Search engine integration (Google, Bing, GitHub)
* And more coming soon...

## Requirements

* Rust 1.70 or higher
* Cargo (Rust's package manager)

## Installation

```bash
# Clone the repository
git clone https://github.com/makalin/gasmask.git
cd gasmask

# Build the project
cargo build --release

# The binary will be available at target/release/gasmask
```

## Documentation

For detailed documentation, including usage examples, configuration options, and best practices, please refer to the [Documentation](docs/DOCUMENTATION.md).

## Usage

```bash
# Basic usage
./target/release/gasmask -d example.com

# Run specific modules
./target/release/gasmask -d example.com -i dns,whois

# Save results to file
./target/release/gasmask -d example.com -o results/example_com

# Use with API keys
./target/release/gasmask -d example.com -i shodan -k YOUR_SHODAN_API_KEY

# Verbose output
./target/release/gasmask -d example.com -v

# Debug mode
./target/release/gasmask -d example.com -D
```

## Configuration

API keys can be stored in an `api_keys.txt` file in the following format:

```
SHODAN_API_KEY=your_key_here
SPYSE_API_KEY=your_key_here
CENSYS_API_ID=your_id_here
CENSYS_API_SECRET=your_secret_here
```

## Credits

Original authors:
* [maldevel](https://github.com/maldevel)
* [mikismaos](https://github.com/mikismaos)
* [xvass](https://github.com/xen0vas)
* [ndamoulianos](https://github.com/ndamoulianos)
* [sbrb](https://github.com/sbrb)

## License

This project is licensed under the GPL-3.0 License - see the LICENSE file for details.
