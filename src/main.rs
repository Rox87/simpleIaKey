use app::config::AppConfig;
use app::ai_client::{fetch_ai, fetch_ai_code, fetch_ai_html};

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
}

fn simulate_copy_and_get_text(enigo: &mut Enigo, clipboard: &mut Clipboard) -> String {
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('x'));
    enigo.key_up(Key::Control);

    thread::sleep(Duration::from_millis(100));

    match clipboard.get_text() {
        Ok(t) => t,
        Err(e) => {
            error!("Erro ao ler clipboard: {}", e);
            String::new()
        }
    }
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

        let mut retry = 3;
        let mut success = false;
        let mut result = String::new();

        while !success && retry > 0 {
            match task {
                TaskType::Melhore => {
                    info!("Melhore task...");
                    let prompt = format!("sem rodeios, retorne o mesmo texto de entrada, melhorado: {}", query);
                    result = fetch_ai(&prompt, &config);
                    if !result.is_empty() { success = true; }
                }
                TaskType::General => {
                    info!("General task...");
                    result = fetch_ai(&query, &config);
                    if !result.is_empty() { success = true; }
                }
                TaskType::Python => {
                    info!("Python Code task...");
                    result = fetch_ai_code(&query, &config);
                    if !result.is_empty() { success = true; }
                }
                TaskType::Html => {
                    info!("HTML task...");
                    result = fetch_ai_html(&query, &config);
                    if !result.is_empty() { success = true; }
                }
            }

            if !success {
                retry -= 1;
                thread::sleep(Duration::from_millis(1000));
            }
        }

        if success {
            simulate_paste(&mut enigo, &mut clipboard, &result);
            info!("Resultado colado com sucesso.");
        } else {
            error!("Falha após as tentativas. Nada foi colado.");
        }
    }
}

fn send_task_on_key(key: RdevKey, sender: &Sender<TaskType>) {
    match key {
        RdevKey::F3 => {
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
        _ => {}
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = AppConfig::load();
    let (sender, receiver) = unbounded();

    let model_info = format!("Geral: {} | Código: {}", config.modelo_geral, config.modelo_codigo);

    thread::spawn(move || {
        worker_loop(receiver, config);
    });

    info!("Listener ativo. Modelos: {}. F3, F8, F9, F10 ativos.", model_info);

    let callback = move |event: Event| {
        if let EventType::KeyPress(key) = event.event_type {
            send_task_on_key(key, &sender);
        }
    };

    if let Err(error) = listen(callback) {
        error!("Erro no listen do teclado: {:?}", error);
    }
}
