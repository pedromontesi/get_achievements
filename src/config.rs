use std::env;

pub struct Config {
    pub api_key: String,
    pub username: String,
    pub port: u16,
    pub base_url: String,
    pub cache_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            api_key: required("API_KEY")?,
            username: required("RA_USERNAME")?,
            port: optional_number("PORT", 3001)?,
            cache_seconds: optional_number("CACHE_SECONDS", 60)?,
            base_url: env::var("RA_BASE_URL")
                .ok()
                .map(|v| v.trim().trim_end_matches('/').to_string())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| "https://retroachievements.org".to_string()),
        })
    }
}

fn required(name: &str) -> Result<String, String> {
    match env::var(name) {
        Ok(v) if !v.trim().is_empty() => Ok(v.trim().to_string()),
        _ => Err(format!(
            "{name} está vazia ou ausente. Preencha {name} no arquivo .env (modelo em .env.example)."
        )),
    }
}

fn optional_number<T: std::str::FromStr>(name: &str, default: T) -> Result<T, String> {
    match env::var(name) {
        Ok(v) if !v.trim().is_empty() => v
            .trim()
            .parse()
            .map_err(|_| format!("{name} inválida: \"{}\".", v.trim())),
        _ => Ok(default),
    }
}
