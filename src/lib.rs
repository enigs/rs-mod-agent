use actix_web::{HttpMessage, HttpRequest};
use once_cell::sync::OnceCell;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use user_agent_parser::UserAgentParser;

/// Global static instance of `UserAgentParser`.
pub static USER_AGENT_PARSER: OnceCell<UserAgentParser> = OnceCell::new();

/// Represents parsed user agent information, including details about the user's device,
/// operating system, browser engine, and CPU architecture.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserAgent {
    pub ip: Option<String>,
    pub product: Product,
    pub os: OS,
    pub device: Device,
    pub cpu: CPU,
    pub engine: Engine,
    pub user_agent: Option<String>
}

/// Retrieves the path to the user agent regex file.
/// Uses the `USER_AGENT_PATH` environment variable, falling back to `./assets/regexes.yaml`.
fn user_agent_path() -> String {
    match std::env::var("USER_AGENT_PATH") {
        Ok(path) => path,
        _ => "./assets/regexes.yaml".to_string()
    }
}

/// Initializes the `UserAgentParser` from the regex file path.
///
/// # Panics
/// If the regex file cannot be loaded, this function will panic.
pub fn init() -> UserAgentParser {
    UserAgentParser::from_path(user_agent_path())
        .unwrap_or_else(|e| panic!("{}", e.to_string()))
}

/// Retrieves the global `UserAgentParser` instance, initializing it if necessary.
pub fn get() -> &'static UserAgentParser {
    USER_AGENT_PARSER.get_or_init(init)
}

/// Parses a user agent string and IP address into a `UserAgent` struct.
///
/// # Arguments
/// - `agent`: The user agent string.
/// - `ip`: The IP address of the client.
///
/// # Returns
/// A `UserAgent` struct containing parsed details.
pub fn parse<T>(agent: T, ip: T) -> UserAgent
where T: ToString
{
    let agent = agent.to_string();
    let ip = ip.to_string();

    let mut user_agent = UserAgent {
        ip: Some(ip),
        ..Default::default()
    };

    let parser = get();

    let product =  parser.parse_product(&agent);
    let os = parser.parse_os(&agent);
    let device = parser.parse_device(&agent);
    let cpu = parser.parse_cpu(&agent);
    let engine = parser.parse_engine(&agent);

    // Set text user agent
    user_agent.user_agent = Some(agent.clone())
        .filter(|s| !s.is_empty());

    // Set product
    user_agent.product.name = product.name.map(|item| item.to_string());
    user_agent.product.major = product.major.map(|item| item.to_string());
    user_agent.product.minor = product.minor.map(|item| item.to_string());
    user_agent.product.patch = product.patch.map(|item| item.to_string());

    // Set os
    user_agent.os.name = os.name.map(|item| item.to_string());
    user_agent.os.major = os.major.map(|item| item.to_string());
    user_agent.os.minor = os.minor.map(|item| item.to_string());
    user_agent.os.patch = os.patch.map(|item| item.to_string());
    user_agent.os.patch_minor = os.patch_minor.map(|item| item.to_string());

    // Set device
    user_agent.device.name = device.name.map(|item| item.to_string());
    user_agent.device.brand = device.brand.map(|item| item.to_string());
    user_agent.device.model = device.model.map(|item| item.to_string());

    // Set architecture
    user_agent.cpu.architecture = cpu.architecture.map(|item| item.to_string());

    // Set engine
    user_agent.engine.name = engine.name.map(|item| item.to_string());
    user_agent.engine.major = engine.major.map(|item| item.to_string());
    user_agent.engine.minor = engine.minor.map(|item| item.to_string());
    user_agent.engine.patch = engine.patch.map(|item| item.to_string());

    user_agent
}


impl UserAgent {
    /// Creates a new default `UserAgent` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Serializes the `UserAgent` struct into JSON format.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self.clone()).unwrap()
    }

    /// Retrieves the `UserAgent` from the `HttpRequest` extensions.
    ///
    /// If the `UserAgent` is not found, returns a default instance.
    pub fn get(req: &HttpRequest) -> Self {
        if let Some(user_agent) = req.extensions().get::<Self>() {
            return user_agent.clone();
        }

        Self::default()
    }
}

/// Represents CPU architecture details.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CPU {
    pub architecture: Option<String>,
}

/// Represents details of the device, such as brand and model.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
}

/// Represents details about the browser engine.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Engine {
    pub name: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub patch: Option<String>,
    pub patch_minor: Option<String>
}

/// Represents details about the operating system.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OS {
    pub name: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub patch: Option<String>,
    pub patch_minor: Option<String>
}

/// Represents details about the browser or application.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub name: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub patch: Option<String>,
}
