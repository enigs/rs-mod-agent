# agent

A Rust library for parsing and handling user agent information in web applications.

## Overview

`agent` provides functionality to parse and extract detailed information from user agent strings. It offers a convenient way to identify browsers, operating systems, devices, and more from HTTP requests.

## Features

- Parse user agent strings into structured data
- Extract device information (brand, model)
- Identify operating systems with version details
- Determine browser/product information
- CPU architecture detection
- Browser engine information
- IP address tracking
- Integration with Actix Web framework

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
agent = { git = "https://github.com/enigs/rs-mod-agent", branch = "main" }
```

## Usage

### Basic Usage

```rust
use agent::{parse, UserAgent};

fn main() {
    // Initialize the parser
    agent::init();
    
    // Parse a user agent string with IP
    let ua_string = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
    let ip = "192.168.1.1";
    
    let user_agent = parse(ua_string, ip);
    
    println!("Browser: {:?}", user_agent.product.name);
    println!("OS: {:?}", user_agent.os.name);
    println!("Device: {:?}", user_agent.device.name);
}
```

### Usage with Actix Web

```rust
use actix_web::{get, web, App, HttpRequest, HttpServer, Responder};
use agent::UserAgent;

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    let user_agent = UserAgent::get(&req);
    
    format!("Hello from {:?} on {:?}!", 
        user_agent.product.name.unwrap_or_else(|| "Unknown".to_string()), 
        user_agent.os.name.unwrap_or_else(|| "Unknown".to_string()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the user agent parser
    agent::init();
    
    HttpServer::new(|| {
        App::new()
            .service(index)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Configuration

The library looks for the user agent regex patterns in the following locations:

1. Path specified in the `USER_AGENT_PATH` environment variable
2. Default path: `./assets/regexes.yaml`

Make sure to have the regex patterns file available at one of these locations.

## API Reference

### Structs

- `UserAgent`: Main struct containing all parsed information
- `Product`: Browser/application details
- `OS`: Operating system information
- `Device`: Device details
- `CPU`: CPU architecture information
- `Engine`: Browser engine details

### Functions

- `init()`: Initialize the user agent parser
- `get()`: Get the global parser instance
- `parse(agent, ip)`: Parse a user agent string and IP address
- `UserAgent::new()`: Create a new default UserAgent instance
- `UserAgent::to_json()`: Serialize UserAgent to JSON
- `UserAgent::get(req)`: Extract UserAgent from an Actix HttpRequest

## Examples

### Extracting Device Information

```rust
use agent::{parse, UserAgent};

fn get_device_info(ua_string: &str, ip: &str) -> String {
    let user_agent = parse(ua_string, ip);
    
    match (user_agent.device.brand, user_agent.device.model) {
        (Some(brand), Some(model)) => format!("{} {}", brand, model),
        (Some(brand), None) => brand,
        _ => "Unknown device".to_string()
    }
}
```

### Working with JSON

```rust
use agent::{parse, UserAgent};
use serde_json::Value;

fn user_agent_to_json(ua_string: &str, ip: &str) -> Value {
    let user_agent = parse(ua_string, ip);
    user_agent.to_json()
}
```