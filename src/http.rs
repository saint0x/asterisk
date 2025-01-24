use crate::error::{Result, AsteriskError};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};
use std::time::Instant;

#[derive(Debug)]
pub struct RequestTiming {
    start: Instant,
    first_byte: Option<Instant>,
    end: Option<Instant>,
}

impl RequestTiming {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            first_byte: None,
            end: None,
        }
    }

    pub fn set_first_byte(&mut self) {
        self.first_byte = Some(Instant::now());
    }

    pub fn set_end(&mut self) {
        self.end = Some(Instant::now());
    }

    pub fn format(&self) -> String {
        let total = self.end.unwrap_or_else(Instant::now).duration_since(self.start);
        let first_byte = self.first_byte.map(|t| t.duration_since(self.start));

        match first_byte {
            Some(fb) => format!(
                "Total: {}ms (First byte: {}ms)",
                total.as_millis(),
                fb.as_millis()
            ),
            None => format!("Total: {}ms", total.as_millis()),
        }
    }
}

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn parse_headers(headers_str: &str, token: Option<&str>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        
        // Add Bearer token if provided
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
                let parts: Vec<&str> = header.split(':').collect();
                if parts.len() != 2 {
                    return Err(AsteriskError::InvalidHeaders);
                }

                let name = HeaderName::from_bytes(parts[0].trim().as_bytes())
                    .map_err(|_| AsteriskError::InvalidHeaders)?;
                let value = HeaderValue::from_str(parts[1].trim())
                    .map_err(|_| AsteriskError::InvalidHeaders)?;

                headers.insert(name, value);
            }
        }

        Ok(headers)
    }

    pub async fn send_request(
        &self,
        url: &str,
        method: &str,
        headers: HeaderMap,
        body: Option<String>,
    ) -> Result<(u16, String, String)> {
        let mut timing = RequestTiming::new();
        
        // Parse HTTP method
        let method = match method.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            _ => return Err(AsteriskError::InvalidHeaders),
        };

        let mut request = self.client.request(method, url);
        request = request.headers(headers);

        // Add body for POST/PUT/PATCH
        if let Some(body) = body {
            request = request
                .header(HeaderName::from_static("content-type"), "application/json")
                .body(body);
        }

        let response = request
            .send()
            .await?;

        timing.set_first_byte();
        
        let status = response.status();
        let body = response.text().await?;
        
        timing.set_end();

        Ok((
            status.as_u16(),
            body,
            timing.format(),
        ))
    }
}
