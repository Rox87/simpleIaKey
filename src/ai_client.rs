use crate::config::AppConfig;
use log::{error, info};
use regex::Regex;
use serde_json::json;
use std::time::Instant;

enum Provider {
    Gemini,
    DeepSeek,
    Unknown,
}

fn detect_provider(model: &str) -> Provider {
    if model.to_lowercase().starts_with("gemin") {
        Provider::Gemini
    } else if model.to_lowercase().starts_with("deeps") {
        Provider::DeepSeek
    } else {
        Provider::Unknown
    }
}

fn call_gemini(api_key: &str, model: &str, prompt: &str) -> String {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );
    let client = reqwest::blocking::Client::new();
    let payload = json!({
        "contents": [{"parts": [{"text": prompt}]}]
    });

    match client.post(&url).json(&payload).send() {
        Ok(res) if res.status().is_success() => {
            if let Ok(json) = res.json::<serde_json::Value>() {
                if let Some(text) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    return text.to_string();
                }
            }
            String::new()
        }
        Ok(res) => {
            error!("Erro Gemini ({}): {:?}", res.status(), res.text());
            String::new()
        }
        Err(e) => {
            error!("Falha Gemini: {}", e);
            String::new()
        }
    }
}

fn call_deepseek(api_key: &str, model: &str, prompt: &str) -> String {
    let url = "https://api.deepseek.com/chat/completions";
    let client = reqwest::blocking::Client::new();
    let payload = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": "Você é um assistente útil e direto. Retorne apenas o resultado solicitado, sem conversas."},
            {"role": "user", "content": prompt}
        ],
        "stream": false
    });

    match client.post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send() {
        Ok(res) if res.status().is_success() => {
            if let Ok(json) = res.json::<serde_json::Value>() {
                if let Some(text) = json["choices"][0]["message"]["content"].as_str() {
                    return text.to_string();
                }
            }
            String::new()
        }
        Ok(res) => {
            error!("Erro DeepSeek ({}): {:?}", res.status(), res.text());
            String::new()
        }
        Err(e) => {
            error!("Falha DeepSeek: {}", e);
            String::new()
        }
    }
}

pub fn fetch_ai(query: &str, config: &AppConfig) -> String {
    let start = Instant::now();
    let model = &config.modelo_geral;
    let provider = detect_provider(model);
    
    let result = match provider {
        Provider::DeepSeek => call_deepseek(&config.deepseek_api_key, model, query),
        Provider::Gemini => call_gemini(&config.gemini_api_key, model, query),
        Provider::Unknown => {
            error!("Provedor desconhecido para o modelo: {}", model);
            String::new()
        }
    };
    
    info!("[{}] Tempo: {:.2}s", model, start.elapsed().as_secs_f64());
    result
}

pub fn fetch_ai_code(query: &str, config: &AppConfig) -> String {
    let start = Instant::now();
    let model = &config.modelo_codigo;
    let provider = detect_provider(model);
    let prompt = format!("sem rodeios, retorne o código python que: {}", query);
    
    let content = match provider {
        Provider::DeepSeek => call_deepseek(&config.deepseek_api_key, model, &prompt),
        Provider::Gemini => call_gemini(&config.gemini_api_key, model, &prompt),
        Provider::Unknown => {
            error!("Provedor desconhecido para o modelo: {}", model);
            String::new()
        }
    };

    info!("[{} CODE] Tempo: {:.2}s", model, start.elapsed().as_secs_f64());

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

pub fn fetch_ai_html(query: &str, config: &AppConfig) -> String {
    let model = &config.modelo_codigo;
    let provider = detect_provider(model);
    let prompt = format!("sem rodeios, retorne o código html de uma página sobre: {}", query);
    
    match provider {
        Provider::DeepSeek => call_deepseek(&config.deepseek_api_key, model, &prompt),
        Provider::Gemini => call_gemini(&config.gemini_api_key, model, &prompt),
        Provider::Unknown => String::new()
    }
}
