use crate::error::{AsteriskError, Result};
use std::collections::HashMap;
use std::env;

/// Resolves environment variables in configuration values
/// Supports $VAR_NAME and ${VAR_NAME} syntax
pub struct EnvironmentResolver {
    cache: HashMap<String, String>,
}

impl EnvironmentResolver {
    /// Creates a new environment resolver with empty cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Resolves environment variables in a string value
    /// Returns error if any referenced variable is not found
    pub fn resolve(&mut self, value: &str) -> Result<String> {
        if !value.contains('$') {
            return Ok(value.to_string());
        }

        let mut result = value.to_string();
        let mut chars = value.chars().peekable();
        let mut current_pos = 0;

        while let Some(ch) = chars.next() {
            if ch == '$' {
                let start_pos = current_pos;
                
                // Handle ${VAR_NAME} syntax
                let (var_name, end_pos) = if chars.peek() == Some(&'{') {
                    chars.next(); // consume '{'
                    let mut var_name = String::new();
                    let mut found_closing = false;
                    
                    for ch in chars.by_ref() {
                        if ch == '}' {
                            found_closing = true;
                            break;
                        }
                        if ch.is_alphanumeric() || ch == '_' {
                            var_name.push(ch);
                        } else {
                            return Err(AsteriskError::InvalidEnvironmentVariable(
                                format!("Invalid character '{}' in environment variable name", ch)
                            ));
                        }
                    }
                    
                    if !found_closing {
                        return Err(AsteriskError::InvalidEnvironmentVariable(
                            "Unterminated environment variable reference (missing '}')".to_string()
                        ));
                    }
                    
                    {
                        let name_len = var_name.len();
                        (var_name, current_pos + name_len + 3) // ${ + name + }
                    }
                } 
                // Handle $VAR_NAME syntax
                else {
                    let mut var_name = String::new();
                    
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' {
                            var_name.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    
                    if var_name.is_empty() {
                        return Err(AsteriskError::InvalidEnvironmentVariable(
                            "Empty environment variable name after '$'".to_string()
                        ));
                    }
                    
                    {
                        let name_len = var_name.len();
                        (var_name, current_pos + name_len + 1) // $ + name
                    }
                };

                // Validate variable name
                if var_name.is_empty() {
                    return Err(AsteriskError::InvalidEnvironmentVariable(
                        "Empty environment variable name".to_string()
                    ));
                }

                if !var_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(AsteriskError::InvalidEnvironmentVariable(
                        format!("Invalid environment variable name: {}", var_name)
                    ));
                }

                // Resolve variable value
                let var_value = self.get_env_var(&var_name)?;
                
                // Replace in result string
                let pattern = if value.chars().nth(start_pos + 1) == Some('{') {
                    format!("${{{}}}", var_name)
                } else {
                    format!("${}", var_name)
                };
                
                result = result.replace(&pattern, &var_value);
                current_pos = end_pos;
            } else {
                current_pos += ch.len_utf8();
            }
        }

        Ok(result)
    }

    /// Gets environment variable value with caching
    fn get_env_var(&mut self, name: &str) -> Result<String> {
        if let Some(cached_value) = self.cache.get(name) {
            return Ok(cached_value.clone());
        }

        match env::var(name) {
            Ok(value) => {
                self.cache.insert(name.to_string(), value.clone());
                Ok(value)
            }
            Err(_) => Err(AsteriskError::MissingEnvironmentVariable(name.to_string())),
        }
    }

    /// Clears the environment variable cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for EnvironmentResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_no_variables() {
        let mut resolver = EnvironmentResolver::new();
        assert_eq!(resolver.resolve("plain_string").unwrap(), "plain_string");
    }

    #[test]
    fn test_simple_variable() {
        env::set_var("TEST_VAR", "test_value");
        let mut resolver = EnvironmentResolver::new();
        assert_eq!(resolver.resolve("$TEST_VAR").unwrap(), "test_value");
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_braced_variable() {
        env::set_var("TEST_VAR", "test_value");
        let mut resolver = EnvironmentResolver::new();
        assert_eq!(resolver.resolve("${TEST_VAR}").unwrap(), "test_value");
        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_variable_in_string() {
        env::set_var("API_HOST", "localhost");
        let mut resolver = EnvironmentResolver::new();
        assert_eq!(
            resolver.resolve("http://${API_HOST}:3000").unwrap(),
            "http://localhost:3000"
        );
        env::remove_var("API_HOST");
    }

    #[test]
    fn test_missing_variable() {
        let mut resolver = EnvironmentResolver::new();
        assert!(resolver.resolve("$NONEXISTENT_VAR").is_err());
    }

    #[test]
    fn test_invalid_variable_name() {
        let mut resolver = EnvironmentResolver::new();
        assert!(resolver.resolve("$INVALID-NAME").is_err());
    }

    #[test]
    fn test_empty_variable_name() {
        let mut resolver = EnvironmentResolver::new();
        assert!(resolver.resolve("$").is_err());
    }

    #[test]
    fn test_unterminated_brace() {
        let mut resolver = EnvironmentResolver::new();
        assert!(resolver.resolve("${UNCLOSED").is_err());
    }
}