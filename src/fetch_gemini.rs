use crate::config::AppConfig;
use log::{error, info};
use regex::Regex;
use serde_json::json;
use std::time::Instant;

// Função auxiliar que chama a API do Gemini via reqwest
fn call_gemini_api(api_key: &str, model: &str, contents: &str) -> String {
    let start = Instant::now();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let client = reqwest::blocking::Client::new();
    let payload = json!({
        "contents": [{
            "parts": [{"text": contents}]
        }]
    });

    let result = client.post(&url).json(&payload).send();

    let duration = start.elapsed();
    info!("[call_gemini_api] Tempo decorrido: {:.2} segundos", duration.as_secs_f64());

    match result {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(json_body) = response.json::<serde_json::Value>() {
                    // Tenta extrair o texto do primeiro candidato
                    if let Some(candidates) = json_body.get("candidates").and_then(|c| c.as_array()) {
                        if let Some(first_candidate) = candidates.first() {
                            if let Some(content) = first_candidate.get("content") {
                                if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                                    if let Some(first_part) = parts.first() {
                                        if let Some(text) = first_part.get("text").and_then(|t| t.as_str()) {
                                            return text.to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                error!("A resposta da API do Gemini não continha o texto esperado.");
                String::new()
            } else {
                error!("Erro na API do Gemini: {}", response.status());
                if let Ok(text) = response.text() {
                    error!("Detalhes do erro: {}", text);
                }
                String::new()
            }
        }
        Err(e) => {
            error!("Falha na requisição para a API do Gemini: {}", e);
            String::new()
        }
    }
}

pub fn fetch_gemini(query: &str, config: &AppConfig) -> String {
    call_gemini_api(&config.gemini_api_key, &config.modelo_geral, query)
}

pub fn fetch_gemini_up(query: &str, config: &AppConfig) -> String {
    call_gemini_api(&config.gemini_api_key, &config.modelo_up, query)
}

pub fn fetch_gemini_html(query: &str, config: &AppConfig) -> String {
    let prompt = format!("sem rodeios, retorne o código html de uma página sobre: {}", query);
    call_gemini_api(&config.gemini_api_key, &config.modelo_html, &prompt)
}

pub fn fetch_gemini_code(query: &str, config: &AppConfig) -> String {
    let prompt = format!("sem rodeios, retorne o código python que: {}", query);
    let content = call_gemini_api(&config.gemini_api_key, &config.modelo_python, &prompt);

    // Extrai todos os blocos de código python usando regex
    let re = Regex::new(r"(?s)```python\n(.*?)```").unwrap();
    let mut matches = vec![];
    for cap in re.captures_iter(&content) {
        if let Some(matched) = cap.get(1) {
            matches.push(matched.as_str().trim().to_string());
        }
    }

    if !matches.is_empty() {
        let mut formatted = format!("# {}\n\n", query);
        for (idx, code) in matches.iter().enumerate() {
            formatted.push_str(&format!("# {}\n{}\n\n", idx + 1, code));
        }
        formatted.trim_end().to_string()
    } else {
        content.trim().to_string()
    }
}
