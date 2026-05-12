use ini::Ini;
use log::error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub gemini_api_key: String,
    pub modelo_geral: String,
    pub modelo_python: String,
    pub modelo_up: String,
    pub modelo_html: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let conf = match Ini::load_from_file("config.ini") {
            Ok(c) => c,
            Err(e) => {
                error!("Erro ao ler config.ini: {}", e);
                // Retornar um valor padrão em caso de erro para não quebrar imediatamente.
                return AppConfig {
                    gemini_api_key: String::new(),
                    modelo_geral: String::from("gemini-1.5-flash"),
                    modelo_python: String::from("gemini-1.5-pro"),
                    modelo_up: String::from("gemini-1.5-pro"),
                    modelo_html: String::from("gemini-1.5-flash"),
                };
            }
        };

        let gemini_api_key = conf
            .section(Some("AI"))
            .and_then(|sec| sec.get("gem"))
            .unwrap_or("")
            .to_string();

        let modelo_geral = conf
            .section(Some("Gemini"))
            .and_then(|sec| sec.get("modelo_geral"))
            .unwrap_or("gemini-1.5-flash")
            .to_string();

        let modelo_python = conf
            .section(Some("Gemini"))
            .and_then(|sec| sec.get("modelo_python"))
            .unwrap_or("gemini-1.5-pro")
            .to_string();

        let modelo_up = conf
            .section(Some("Gemini"))
            .and_then(|sec| sec.get("modelo_up"))
            .unwrap_or("gemini-1.5-pro")
            .to_string();

        let modelo_html = conf
            .section(Some("Gemini"))
            .and_then(|sec| sec.get("modelo_html"))
            .unwrap_or("gemini-1.5-flash")
            .to_string();

        AppConfig {
            gemini_api_key,
            modelo_geral,
            modelo_python,
            modelo_up,
            modelo_html,
        }
    }
}
