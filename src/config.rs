use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub modelo_geral: String,
    pub modelo_codigo: String,
    pub gemini_api_key: String,
    pub deepseek_api_key: String,
    pub grok_api_key: String,
}

impl AppConfig {
    pub fn load() -> Self {
        dotenv().ok();

        AppConfig {
            modelo_geral: env::var("MODELO_GERAL").unwrap_or_default().to_string(),
            modelo_codigo: env::var("MODELO_CODIGO").unwrap_or_default(),
            gemini_api_key: env::var("GEMINI_API_KEY").unwrap_or_default(),
            deepseek_api_key: env::var("DEEPSEEK_API_KEY").unwrap_or_default(),
            grok_api_key: env::var("GROK_API_KEY").unwrap_or_default(),
        }
    }
}
