use crate::config::AppConfig;
use log::{error, info};
use regex::Regex;
use serde_json::json;
use std::time::Instant;

enum Provider {
    Gemini,
    DeepSeek,
    Grok,
    Unknown,
}

fn detect_provider(model: &str) -> Provider {
    if model.to_lowercase().starts_with("gemin") {
        Provider::Gemini
    } else if model.to_lowercase().starts_with("deeps") {
        Provider::DeepSeek
    } else if model.to_lowercase().starts_with("grok") {
        Provider::Grok
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

fn call_grok(api_key: &str, model_config: &str, prompt: &str) -> String {
    // Faz o split uma única vez para evitar a perda da string original
    let parts: Vec<&str> = model_config.split('_').collect();
    
    // Define o modelo (padrão usa a string inteira se não houver '_')
    let model_name = parts[0];
    
    // Se houver uma segunda parte e for "high", usamos "text", caso contrário "none"
    let reasoning_format = if parts.len() > 1 && parts[1] != "none" {
        Some(parts[1])
    } else {
        None
    };

    let url = "https://api.x.ai/v1/chat/completions";
    let client = reqwest::blocking::Client::new();
    
    let mut payload = json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": "Você é um assistente útil e direto. Retorne apenas o resultado solicitado, sem conversas."},
            {"role": "user", "content": prompt}
        ],
        "stream": false
    });

    //if let Some(format) = reasoning_format {
    payload["reasoning_format"] = json!(reasoning_format);
    //}

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
            error!("Erro Grok ({}): {:?}", res.status(), res.text());
            String::new()
        }
        Err(e) => {
            error!("Falha Grok: {}", e);
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
        Provider::Grok => call_grok(&config.grok_api_key, model, query),
        Provider::Unknown => {
            error!("Provedor desconhecido para o modelo: {}", model);
            String::new()
        }
    };
    
    info!("[{}] Tempo: {:.2}s", model, start.elapsed().as_secs_f64());
    result
}


pub fn comment_char(content: &str) -> &'static str {
    if content.contains("```python") {
        "#"
    } else {
        "//"
    }
}

pub fn fetch_ai_code(query: &str, config: &AppConfig,reasoning_effort: &str) -> String {
    let start = Instant::now();
    
    let base_model = &config.modelo_codigo;
    let provider = detect_provider(base_model);
    let prompt = format!("sem rodeios, retorne o código {}", query);
    
    // Modifica a string do modelo para incluir o formato esperado pelas funções de chamada
    // Exemplo: "grok-3" vira "grok-3_none"
    let model_with_config = format!("{}_{}", base_model, reasoning_effort);
    
    let content = match provider {
        // Passamos 'model_with_config' para o Grok saber se aplica o reasoning no payload dele
        Provider::Grok => call_grok(&config.grok_api_key, &model_with_config, &prompt),
        
        // Se a sua função do Gemini/DeepSeek aceitar o modelo direto e você preferir tratar lá dentro:
        Provider::Gemini => call_gemini(&config.gemini_api_key, base_model, &prompt),
        Provider::DeepSeek => call_deepseek(&config.deepseek_api_key, base_model, &prompt),
        
        Provider::Unknown => {
            error!("Provedor desconhecido para o modelo: {}", base_model);
            String::new()
        }
    };
    
    info!("[{} CODE] Tempo: {:.2}s", base_model, start.elapsed().as_secs_f64());

    // CORREÇÃO: O Regex agora está em uma linha contínua perfeita
    let re = Regex::new(r"(?s)```[a-zA-Z0-9_-]*\s*\n(?:# )?(.*?)```").unwrap();
    let mut matches = vec![];
    for cap in re.captures_iter(&content) {
        if let Some(matched) = cap.get(1) {
            matches.push(matched.as_str().trim().to_string());
        }
    }

    let mychar = comment_char(&content);
            
    if !matches.is_empty() {
    let mut formatted = format!("{} {} \n", mychar, query.replace("\n", &format!("\n{} ", mychar)));
        for (idx, code) in matches.iter().enumerate() {
            formatted.push_str(&format!("{}{}\n{}\n\n",mychar,idx+1, code));
        }
        formatted.trim_end().to_string()
    } else {
        content.trim().to_string()
    }
}

pub fn fetch_ai_code_low(query: &str, config: &AppConfig) -> String {
    fetch_ai_code(query, config, "none")
}

pub fn fetch_ai_code_high(query: &str, config: &AppConfig) -> String {
    fetch_ai_code(query, config, "high")
}