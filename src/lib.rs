use actix_web::{HttpMessage, HttpRequest};
use blake3::Hasher;
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
    pub fingerprint: Option<String>,
    pub hash: Option<String>,
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

    // Generate fingerprint and hash
    user_agent.fingerprint = user_agent.fingerprint();
    user_agent.hash = user_agent.hash();

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

    /// Creates a robust fingerprint using multiple attributes of the user agent.
    ///
    /// The fingerprint combines various stable aspects of the user agent to create
    /// a consistent identifier that is difficult to forge or debug.
    ///
    /// # Returns
    /// An Option containing the fingerprint string, or None if insufficient data is available.
    pub fn fingerprint(&self) -> Option<String> {
        // Get the user agent string, using ? to return None if not available
        let ua = self.user_agent.as_ref()?;

        // Create feature vectors for different components
        let mut feature_parts = Vec::new();

        // Browser component with weighting factors
        if let Some(browser_name) = &self.product.name {
            // Add browser name with position marker
            feature_parts.push(format!("b:{}", browser_name));

            // Add version parts with more specific markers
            if let Some(major) = &self.product.major {
                feature_parts.push(format!("bv:{}", major));

                if let Some(minor) = &self.product.minor {
                    feature_parts.push(format!("bvm:{}.{}", major, minor));
                }
            }
        }

        // OS component with weighting factors
        if let Some(os_name) = &self.os.name {
            // Add OS name with position marker
            feature_parts.push(format!("o:{}", os_name));

            // Add version parts with more specific markers
            if let Some(major) = &self.os.major {
                feature_parts.push(format!("ov:{}", major));

                if let Some(minor) = &self.os.minor {
                    feature_parts.push(format!("ovm:{}.{}", major, minor));
                }
            }
        }

        // Device component
        if let Some(device_name) = &self.device.name {
            feature_parts.push(format!("d:{}", device_name));

            if let Some(brand) = &self.device.brand {
                feature_parts.push(format!("db:{}", brand));
            }

            if let Some(model) = &self.device.model {
                feature_parts.push(format!("dm:{}", model));
            }
        }

        // CPU architecture
        if let Some(arch) = &self.cpu.architecture {
            feature_parts.push(format!("c:{}", arch));
        }

        // Engine component
        if let Some(engine_name) = &self.engine.name {
            feature_parts.push(format!("e:{}", engine_name));

            if let Some(major) = &self.engine.major {
                feature_parts.push(format!("ev:{}", major));
            }
        }

        // Add special signature components derived from the raw user agent
        // Extract unique patterns from the user agent

        // Length characteristics (highly stable)
        feature_parts.push(format!("l:{}", ua.len()));

        // Character distribution characteristics
        let digits = ua.chars().filter(|c| c.is_ascii_digit()).count();
        let symbols = ua.chars().filter(|c| !c.is_alphanumeric()).count();
        feature_parts.push(format!("d:{}", digits));
        feature_parts.push(format!("s:{}", symbols));

        // Word pattern analysis (stable across same browser family)
        let word_count = ua.split_whitespace().count();
        feature_parts.push(format!("w:{}", word_count));

        // Feature detection (browser capabilities)
        if ua.contains("Mobile") {
            feature_parts.push("fm:1".to_string());
        }

        if ua.contains("AppleWebKit") {
            feature_parts.push("faw:1".to_string());
        }

        if ua.contains("Gecko") {
            feature_parts.push("fg:1".to_string());
        }

        if ua.contains("Chrome") {
            feature_parts.push("fc:1".to_string());
        }

        if ua.contains("Safari") && !ua.contains("Chrome") {
            feature_parts.push("fs:1".to_string());
        }

        if ua.contains("Firefox") {
            feature_parts.push("ff:1".to_string());
        }

        if ua.contains("Edge") || ua.contains("Edg/") {
            feature_parts.push("fe:1".to_string());
        }

        if ua.contains("MSIE") || ua.contains("Trident") {
            feature_parts.push("fi:1".to_string());
        }

        if ua.contains("Win") {
            feature_parts.push("fow:1".to_string());
        } else if ua.contains("Mac") {
            feature_parts.push("fom:1".to_string());
        } else if ua.contains("Linux") {
            feature_parts.push("fol:1".to_string());
        } else if ua.contains("Android") {
            feature_parts.push("foa:1".to_string());
        } else if ua.contains("iOS") || ua.contains("iPhone") || ua.contains("iPad") {
            feature_parts.push("foi:1".to_string());
        }

        // Add IP subnet information if available (only use network portion for stability)
        if let Some(ip) = &self.ip {
            if ip.contains('.') {
                // IPv4 address - use first two octets only (network portion)
                let parts: Vec<&str> = ip.split('.').collect();
                if parts.len() >= 2 {
                    feature_parts.push(format!("ip4:{}.{}", parts[0], parts[1]));
                }
            } else if ip.contains(':') {
                // IPv6 address - use first four segments only
                let parts: Vec<&str> = ip.split(':').collect();
                if parts.len() >= 4 {
                    feature_parts.push(format!("ip6:{}", parts[0..4].join(":")));
                }
            }
        }

        // Sort to ensure consistent ordering
        feature_parts.sort();

        // Combine parts with a non-obvious separator
        let features = feature_parts.join("&%&");

        // Apply cryptographic hashing to create the final fingerprint
        let mut hasher = Hasher::new();
        hasher.update(features.as_bytes());

        // Apply a secondary hash to make reverse-engineering more difficult
        let primary_hash = hasher.finalize().to_hex().to_string();
        let mut secondary_hasher = Hasher::new();
        secondary_hasher.update(primary_hash.as_bytes());

        Some(secondary_hasher.finalize().to_hex().to_string())
    }

    /// Creates a hash suitable for use as a family_id by first normalizing the user agent data.
    ///
    /// This function first creates a normalized string representation of the user agent
    /// that only includes non-empty components separated by pipes. This normalized
    /// representation is then hashed.
    ///
    /// # Returns
    /// An Option containing a string representation of the hash (suitable for family_id),
    /// or None if the user agent string is not available.
    pub fn hash(&self) -> Option<String> {
        self.user_agent.as_ref().map(|ua_string| {
            let normalized_ua = self.normalized_string_internal(ua_string);

            // Hash the normalized representation
            let mut hasher = Hasher::new();
            hasher.update(normalized_ua.as_bytes());
            hasher.finalize().to_hex().to_string()
        })
    }

    /// Returns the normalized string representation of the user agent.
    ///
    /// This function creates a normalized string that only includes
    /// non-empty components separated by pipes.
    ///
    /// # Returns
    /// An Option containing the normalized string representation,
    /// or None if the user agent string is not available.
    pub fn normalized_string(&self) -> Option<String> {
        self.user_agent.as_ref().map(|ua_string| {
            self.normalized_string_internal(ua_string)
        })
    }

    // Internal helper method to create the normalized string
    fn normalized_string_internal(&self, ua_string: &str) -> String {
        // Normalize the Product (browser) component
        let browser_parts: Vec<&str> = [
            self.product.name.as_deref(),
            self.product.major.as_deref(),
            self.product.minor.as_deref(),
            self.product.patch.as_deref()
        ].into_iter()
            .flatten()
            .collect();

        let browser_str = if !browser_parts.is_empty() {
            browser_parts.join(".")
        } else {
            String::new()
        };

        // Normalize the OS component
        let os_parts: Vec<&str> = [
            self.os.name.as_deref(),
            self.os.major.as_deref(),
            self.os.minor.as_deref(),
            self.os.patch.as_deref(),
            self.os.patch_minor.as_deref()
        ].into_iter()
            .flatten()
            .collect();

        let os_str = if !os_parts.is_empty() {
            os_parts.join(".")
        } else {
            String::new()
        };

        // Normalize the Device component
        let device_parts: Vec<&str> = [
            self.device.name.as_deref(),
            self.device.brand.as_deref(),
            self.device.model.as_deref()
        ].into_iter()
            .flatten()
            .collect();

        let device_str = if !device_parts.is_empty() {
            device_parts.join(".")
        } else {
            String::new()
        };

        // Create normalized sections
        let mut sections = Vec::new();

        if !browser_str.is_empty() {
            sections.push(browser_str);
        }

        if !os_str.is_empty() {
            sections.push(os_str);
        }

        if !device_str.is_empty() {
            sections.push(device_str);
        }

        // Always include the user agent string as the last section
        sections.push(ua_string.to_string());

        // Join sections with pipe character
        sections.join("|")
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