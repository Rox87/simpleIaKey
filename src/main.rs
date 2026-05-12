use app::config::AppConfig;
use app::fetch_gemini::{fetch_gemini, fetch_gemini_code, fetch_gemini_html, fetch_gemini_up};

use arboard::Clipboard;
use crossbeam_channel::{unbounded, Sender};
use enigo::{Enigo, KeyboardControllable, Key};
use log::{error, info};
use rdev::{listen, Event, EventType, Key as RdevKey};
use std::thread;
use std::time::Duration;

enum TaskType {
    Melhore,
    General,
    Python,
    Html,
    Up,
}

fn simulate_copy_and_get_text(enigo: &mut Enigo, clipboard: &mut Clipboard) -> String {
    // Simula Ctrl+X (recortar)
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('x'));
    enigo.key_up(Key::Control);

    thread::sleep(Duration::from_millis(100)); // Pequena pausa para o recorte funcionar

    // Lê a área de transferência
    let text = match clipboard.get_text() {
        Ok(t) => t,
        Err(e) => {
            error!("Erro ao ler clipboard: {}", e);
            String::new()
        }
    };

    thread::sleep(Duration::from_millis(50));

    // Escreve "Processando"
    if let Err(e) = clipboard.set_text("Processando") {
        error!("Erro ao definir clipboard para Processando: {}", e);
    }

    thread::sleep(Duration::from_millis(50));

    // Simula Ctrl+V (colar o "Processando")
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('v'));
    enigo.key_up(Key::Control);

    thread::sleep(Duration::from_millis(50));

    // Seleciona o "Processando" para a esquerda (Ctrl+Shift+Left)
    enigo.key_down(Key::Control);
    enigo.key_down(Key::Shift);
    enigo.key_click(Key::LeftArrow);
    enigo.key_up(Key::Shift);
    enigo.key_up(Key::Control);

    thread::sleep(Duration::from_millis(50));

    text
}

fn simulate_paste(enigo: &mut Enigo, clipboard: &mut Clipboard, text: &str) {
    if let Err(e) = clipboard.set_text(text) {
        error!("Erro ao definir clipboard com resultado: {}", e);
    }
    thread::sleep(Duration::from_millis(50));

    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('v'));
    enigo.key_up(Key::Control);

    thread::sleep(Duration::from_millis(50));
}

fn worker_loop(receiver: crossbeam_channel::Receiver<TaskType>, config: AppConfig) {
    let mut enigo = Enigo::new();
    let mut clipboard = Clipboard::new().expect("Não foi possível inicializar o Clipboard");

    for task in receiver {
        let query = simulate_copy_and_get_text(&mut enigo, &mut clipboard);

        if query.is_empty() {
            info!("Nenhum texto selecionado ou erro ao ler o clipboard.");
            continue;
        }

        info!("Query: {}", query);

        let mut retry = 10;
        let mut success = false;
        let mut result = String::new();

        while !success && retry > 0 {
            match task {
                TaskType::Melhore => {
                    info!("Melhore call!");
                    let prompt = format!("sem rodeios, retorne o mesmo texto de entrada, melhorado: {}", query);
                    result = fetch_gemini(&prompt, &config);
                    if !result.is_empty() { success = true; }
                }
                TaskType::General => {
                    info!("General gemini call!");
                    result = fetch_gemini(&query, &config);
                    if !result.is_empty() { success = true; }
                }
                TaskType::Python => {
                    info!("Codigo python call!");
                    result = fetch_gemini_code(&query, &config);
                    if !result.is_empty() { success = true; }
                }
                TaskType::Html => {
                    info!("HTML call!");
                    result = fetch_gemini_html(&query, &config);
                    if !result.is_empty() { success = true; }
                }
                TaskType::Up => {
                    info!("General gemini UP call!");
                    result = fetch_gemini_up(&query, &config);
                    if !result.is_empty() { success = true; }
                }
            }

            if !success {
                retry -= 1;
                thread::sleep(Duration::from_millis(500));
            }
        }

        if success {
            simulate_paste(&mut enigo, &mut clipboard, &result);
            info!("Resultado processado e colado com sucesso.");
        } else {
            error!("Falha após as tentativas. Nada foi colado.");
        }
    }
}

fn send_task_on_key(key: RdevKey, sender: &Sender<TaskType>) {
    match key {
        RdevKey::F2 => {
            let _ = sender.send(TaskType::Melhore);
        }
        RdevKey::F8 => {
            let _ = sender.send(TaskType::General);
        }
        RdevKey::F9 => {
            let _ = sender.send(TaskType::Python);
        }
        RdevKey::F10 => {
            let _ = sender.send(TaskType::Html);
        }
        // Na falta de uma tecla definida explícita para on_activate_up,
        // mas sendo uma função, podemos associar F11 ou ignorá-la.
        // Baseado na revisão, removemos o atalho para UP para nos ater estritamente ao F2, F8, F9, F10 do python.
        _ => {}
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = AppConfig::load();
    let (sender, receiver) = unbounded();

    // Spawn do worker thread
    thread::spawn(move || {
        worker_loop(receiver, config);
    });

    info!("Listener ativo. Pressione F2 (melhore), F8 (general), F9 (python) ou F10 (html) para enviar consulta ao Gemini.");

    // Callback para eventos rdev
    let callback = move |event: Event| {
        if let EventType::KeyPress(key) = event.event_type {
            send_task_on_key(key, &sender);
        }
    };

    if let Err(error) = listen(callback) {
        error!("Erro no listen do teclado: {:?}", error);
    }
}
