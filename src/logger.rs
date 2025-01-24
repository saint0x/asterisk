use colored::*;

pub struct Logger {
    verbose: bool,
}

impl Logger {
    pub fn new(verbose: bool) -> Self {
        Logger { verbose }
    }

    pub fn response(&self, status: u16, timing: &str, body: &str) {
        let status_color = match status {
            200..=299 => status.to_string().green(),
            300..=399 => status.to_string().yellow(),
            _ => status.to_string().red(),
        };

        if self.verbose {
            println!("\n{}", "Response Details:".bold());
            println!("{} {}", "Status Code:".bold(), status_color);
            println!("{} {}", "Status Text:".bold(), self.status_text(status));
            println!("{} {}", "Performance:".bold(), timing);
            println!("\n{}", "Response Body:".bold());
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_else(|_| body.to_string()));
            } else {
                println!("{}", body);
            }
        } else {
            println!("\n{} {}", "Status:".bold(), status_color);
            println!("{} {}", "Timing:".bold(), timing);
            println!("{} {}", "Response:".bold(), body);
        }
    }

    fn status_text(&self, status: u16) -> String {
        match status {
            200 => "OK".green(),
            201 => "Created".green(),
            204 => "No Content".green(),
            301 => "Moved Permanently".yellow(),
            302 => "Found".yellow(),
            304 => "Not Modified".yellow(),
            400 => "Bad Request".red(),
            401 => "Unauthorized".red(),
            403 => "Forbidden".red(),
            404 => "Not Found".red(),
            500 => "Internal Server Error".red(),
            502 => "Bad Gateway".red(),
            503 => "Service Unavailable".red(),
            _ => "Unknown Status".red(),
        }.to_string()
    }
}
