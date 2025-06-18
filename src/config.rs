use crate::environment::EnvironmentResolver;
use crate::error::{AsteriskError, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Configuration file name to search for in project directories
const CONFIG_FILENAME: &str = "asterisk.config";

/// Default configuration template for initialization
const DEFAULT_CONFIG_TEMPLATE: &str = r#"# Asterisk Configuration
# Project-level configuration for API testing

# Default profile to use when none specified
default_profile = "dev"

# Profile definitions
[profiles.dev]
url = "http://localhost:3000"
# token = "$DEV_API_KEY"  # Reference environment variable
# headers = ["X-Environment:development"]

[profiles.staging]
url = "https://staging-api.example.com"
# token = "$STAGING_API_KEY"
# headers = ["X-Environment:staging", "X-Version:1.0"]

[profiles.production]
url = "https://api.example.com"
# token = "$PROD_API_KEY"
# headers = ["X-Environment:production", "X-Version:1.0"]
"#;

/// Raw configuration structure as parsed from TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawConfig {
    /// Default profile to use when none specified
    pub default_profile: Option<String>,
    
    /// Map of profile name to profile configuration
    pub profiles: HashMap<String, RawProfile>,
}

/// Raw profile configuration before environment variable resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawProfile {
    /// Base URL for API requests
    pub url: String,
    
    /// Bearer token (may contain environment variable references)
    pub token: Option<String>,
    
    /// HTTP headers in "key:value" format
    pub headers: Option<Vec<String>>,
    
    /// Enable verbose output by default
    pub verbose: Option<bool>,
}

/// Resolved configuration with environment variables expanded
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// Base URL for API requests
    pub url: String,
    
    /// Bearer token (resolved)
    pub token: Option<String>,
    
    /// HTTP headers as HeaderMap
    pub headers: HeaderMap,
    
    /// Enable verbose output
    pub verbose: bool,
}

/// Configuration manager that handles loading, parsing, and resolving configurations
pub struct ConfigManager {
    environment_resolver: EnvironmentResolver,
}

impl ConfigManager {
    /// Creates a new configuration manager
    pub fn new() -> Self {
        Self {
            environment_resolver: EnvironmentResolver::new(),
        }
    }

    /// Loads and resolves configuration for the current working directory
    /// Applies CLI overrides and returns the final resolved configuration
    pub fn load_resolved_config(
        &mut self,
        profile_override: Option<&str>,
        url_override: Option<&str>,
        token_override: Option<&str>,
        headers_override: Option<&str>,
        verbose_override: bool,
    ) -> Result<ResolvedConfig> {
        // Find and load project configuration
        let raw_config = match self.find_and_load_config()? {
            Some(config) => config,
            None => {
                // No config file found, use defaults with CLI overrides
                return Ok(ResolvedConfig {
                    url: url_override.unwrap_or("http://localhost:3000").to_string(),
                    token: token_override.map(|t| t.to_string()),
                    headers: self.parse_headers_string(headers_override.unwrap_or(""), token_override)?,
                    verbose: verbose_override,
                });
            }
        };

        // Determine which profile to use
        let profile_name = profile_override
            .or(raw_config.default_profile.as_deref())
            .unwrap_or("dev");

        // Get the specified profile
        let raw_profile = raw_config.profiles.get(profile_name)
            .ok_or_else(|| AsteriskError::Config(
                format!("Profile '{}' not found in configuration", profile_name)
            ))?;

        // Resolve environment variables in profile
        let resolved_url = if let Some(url_override) = url_override {
            url_override.to_string()
        } else {
            self.environment_resolver.resolve(&raw_profile.url)?
        };

        let resolved_token = if let Some(token_override) = token_override {
            Some(token_override.to_string())
        } else if let Some(ref token) = raw_profile.token {
            Some(self.environment_resolver.resolve(token)?)
        } else {
            None
        };

        // Parse headers with CLI override taking precedence
        let headers = if let Some(headers_override) = headers_override {
            self.parse_headers_string(headers_override, resolved_token.as_deref())?
        } else {
            self.parse_profile_headers(raw_profile, resolved_token.as_deref())?
        };

        // Determine verbose setting
        let verbose = verbose_override || raw_profile.verbose.unwrap_or(false);

        Ok(ResolvedConfig {
            url: resolved_url,
            token: resolved_token,
            headers,
            verbose,
        })
    }

    /// Finds asterisk.config by walking up the directory tree
    /// Returns the path to the config file if found
    pub fn find_config_file(&self) -> Option<PathBuf> {
        let mut current_dir = std::env::current_dir().ok()?;
        
        loop {
            let config_path = current_dir.join(CONFIG_FILENAME);
            if config_path.exists() {
                return Some(config_path);
            }

            // Stop at git root if present
            if current_dir.join(".git").exists() {
                break;
            }

            // Move up one directory
            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => break, // Reached filesystem root
            }
        }

        None
    }

    /// Loads and parses configuration from file
    fn find_and_load_config(&self) -> Result<Option<RawConfig>> {
        let config_path = match self.find_config_file() {
            Some(path) => path,
            None => return Ok(None),
        };

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| AsteriskError::Config(
                format!("Failed to read config file {}: {}", config_path.display(), e)
            ))?;

        let raw_config: RawConfig = toml::from_str(&config_content)
            .map_err(|e| AsteriskError::Config(
                format!("Failed to parse config file {}: {}", config_path.display(), e)
            ))?;

        // Validate configuration
        self.validate_config(&raw_config)?;

        Ok(Some(raw_config))
    }

    /// Validates the loaded configuration
    fn validate_config(&self, config: &RawConfig) -> Result<()> {
        if config.profiles.is_empty() {
            return Err(AsteriskError::Config(
                "Configuration must contain at least one profile".to_string()
            ));
        }

        // Validate default profile exists if specified
        if let Some(ref default_profile) = config.default_profile {
            if !config.profiles.contains_key(default_profile) {
                return Err(AsteriskError::Config(
                    format!("Default profile '{}' not found in profiles", default_profile)
                ));
            }
        }

        // Validate each profile
        for (name, profile) in &config.profiles {
            if profile.url.is_empty() {
                return Err(AsteriskError::Config(
                    format!("Profile '{}' must have a non-empty URL", name)
                ));
            }

            // Validate header format if present
            if let Some(ref headers) = profile.headers {
                for header in headers {
                    if !header.contains(':') {
                        return Err(AsteriskError::Config(
                            format!("Invalid header format in profile '{}': '{}'. Expected 'key:value'", name, header)
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Parses headers from profile configuration
    fn parse_profile_headers(&mut self, profile: &RawProfile, token: Option<&str>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Add Bearer token if present
        if let Some(token) = token {
            headers.insert(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|_| AsteriskError::InvalidHeaders)?,
            );
        }

        // Add profile headers
        if let Some(ref header_strings) = profile.headers {
            for header_str in header_strings {
                let resolved_header = self.environment_resolver.resolve(header_str)?;
                self.parse_single_header(&resolved_header, &mut headers)?;
            }
        }

        Ok(headers)
    }

    /// Parses headers from CLI string format
    fn parse_headers_string(&mut self, headers_str: &str, token: Option<&str>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Add Bearer token if present
        if let Some(token) = token {
            headers.insert(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|_| AsteriskError::InvalidHeaders)?,
            );
        }

        // Parse custom headers
        if !headers_str.is_empty() {
            for header in headers_str.split(',') {
                let resolved_header = self.environment_resolver.resolve(header.trim())?;
                self.parse_single_header(&resolved_header, &mut headers)?;
            }
        }

        Ok(headers)
    }

    /// Parses a single header in "key:value" format
    fn parse_single_header(&self, header_str: &str, headers: &mut HeaderMap) -> Result<()> {
        let parts: Vec<&str> = header_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AsteriskError::InvalidHeaders);
        }

        let name = HeaderName::from_bytes(parts[0].trim().as_bytes())
            .map_err(|_| AsteriskError::InvalidHeaders)?;
        let value = HeaderValue::from_str(parts[1].trim())
            .map_err(|_| AsteriskError::InvalidHeaders)?;

        headers.insert(name, value);
        Ok(())
    }

    /// Creates a default configuration file in the current directory
    pub fn create_default_config(&self) -> Result<PathBuf> {
        let config_path = std::env::current_dir()
            .map_err(|e| AsteriskError::Config(format!("Cannot get current directory: {}", e)))?
            .join(CONFIG_FILENAME);

        if config_path.exists() {
            return Err(AsteriskError::Config(
                format!("Configuration file already exists: {}", config_path.display())
            ));
        }

        fs::write(&config_path, DEFAULT_CONFIG_TEMPLATE)
            .map_err(|e| AsteriskError::Config(
                format!("Failed to create config file: {}", e)
            ))?;

        Ok(config_path)
    }

    /// Returns the current resolved configuration as a display string
    pub fn format_current_config(&mut self, profile: Option<&str>) -> Result<String> {
        let config = self.load_resolved_config(profile, None, None, None, false)?;
        
        let mut output = String::new();
        output.push_str(&format!("URL: {}\n", config.url));
        
        if let Some(ref token) = config.token {
            // Mask token for security
            let masked_token = if token.len() > 8 {
                format!("{}...{}", &token[..4], &token[token.len()-4..])
            } else {
                "***".to_string()
            };
            output.push_str(&format!("Token: {}\n", masked_token));
        } else {
            output.push_str("Token: None\n");
        }
        
        output.push_str(&format!("Verbose: {}\n", config.verbose));
        
        if !config.headers.is_empty() {
            output.push_str("Headers:\n");
            for (name, value) in &config.headers {
                if name == "authorization" {
                    output.push_str("  authorization: Bearer ***\n");
                } else {
                    output.push_str(&format!("  {}: {}\n", name, value.to_str().unwrap_or("***")));
                }
            }
        }
        
        Ok(output)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_parsing() {
        let config_content = r#"
default_profile = "dev"

[profiles.dev]
url = "http://localhost:3000"
token = "$DEV_TOKEN"
headers = ["X-Environment:development"]
verbose = true

[profiles.prod]
url = "https://api.example.com"
token = "$PROD_TOKEN"
"#;

        let config: RawConfig = toml::from_str(config_content).unwrap();
        assert_eq!(config.default_profile, Some("dev".to_string()));
        assert_eq!(config.profiles.len(), 2);
        assert_eq!(config.profiles["dev"].url, "http://localhost:3000");
        assert_eq!(config.profiles["dev"].token, Some("$DEV_TOKEN".to_string()));
    }

    #[test]
    fn test_config_validation() {
        let manager = ConfigManager::new();
        
        // Valid config
        let config = RawConfig {
            default_profile: Some("dev".to_string()),
            profiles: {
                let mut profiles = HashMap::new();
                profiles.insert("dev".to_string(), RawProfile {
                    url: "http://localhost:3000".to_string(),
                    token: None,
                    headers: None,
                    verbose: None,
                });
                profiles
            },
        };
        assert!(manager.validate_config(&config).is_ok());

        // Invalid: missing default profile
        let config = RawConfig {
            default_profile: Some("nonexistent".to_string()),
            profiles: {
                let mut profiles = HashMap::new();
                profiles.insert("dev".to_string(), RawProfile {
                    url: "http://localhost:3000".to_string(),
                    token: None,
                    headers: None,
                    verbose: None,
                });
                profiles
            },
        };
        assert!(manager.validate_config(&config).is_err());
    }

    #[test]
    fn test_create_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        
        env::set_current_dir(temp_dir.path()).unwrap();
        
        let manager = ConfigManager::new();
        let config_path = manager.create_default_config().unwrap();
        
        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("default_profile"));
        assert!(content.contains("[profiles.dev]"));
        
        env::set_current_dir(original_dir).unwrap();
    }
}