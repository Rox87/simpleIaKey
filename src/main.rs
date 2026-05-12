
#![windows_subsystem = "windows"]

use jKey::config::AppConfig;
use jKey::ai_client::{fetch_ai, fetch_ai_code, fetch_ai_html};

use arboard::Clipboard;
use crossbeam_channel::{unbounded, Sender};
use enigo::{Enigo, KeyboardControllable, Key};
use log::{error, info};
use rdev::{listen, Event, EventType, Key as RdevKey};
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io;

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, Icon,
};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use sysinfo::System;
// use image::GenericImageView;

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

fn load_icon(path: &std::path::Path) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

fn main() {
    // Lógica para fechar instância anterior se houver
    let mut system = System::new_all();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    let current_pid = std::process::id();

    for (pid, process) in system.processes() {
        // Verifica se o nome do processo é jKey (ou jKey.exe no Windows)
        let name = process.name();
        if (name == "jKey.exe" || name == "jKey") && pid.to_string() != current_pid.to_string() {
            info!("Instância anterior detectada (PID: {}). Encerrando para reiniciar...", pid);
            process.kill();
            // Pequena pausa para garantir que o SO libere os recursos da instância anterior
            thread::sleep(Duration::from_millis(500));
        }
    }

    // Configura o hook de pânico para manter a janela aberta em caso de erro fatal
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = match panic_info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Erro desconhecido",
            },
        };

        let location = panic_info.location()
            .map(|l| format!(" em {}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_default();

        eprintln!("\n==================================================");
        eprintln!("ERRO CRÍTICO NA APLICAÇÃO");
        eprintln!("Detalhes: {}{}", msg, location);
        eprintln!("==================================================");
        eprintln!("\nPressione ENTER para fechar esta janela...");

        let mut buffer = String::new();
        let _ = io::stdin().read_line(&mut buffer);
    }));

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = AppConfig::load();
    let (sender, receiver) = unbounded();
    let suspended = Arc::new(AtomicBool::new(false));

    let model_info = format!("Geral: {} | Código: {}", config.modelo_geral, config.modelo_codigo);

    // Thread para o loop de processamento
    thread::spawn(move || {
        worker_loop(receiver, config);
    });

    // Thread para o listener do teclado
    let sender_clone = sender.clone();
    let suspended_clone = suspended.clone();
    thread::spawn(move || {
        let callback = move |event: Event| {
            if !suspended_clone.load(Ordering::SeqCst) {
                if let EventType::KeyPress(key) = event.event_type {
                    send_task_on_key(key, &sender_clone);
                }
            }
        };

        if let Err(error) = listen(callback) {
            error!("Erro no listen do teclado: {:?}", error);
        }
    });

    info!("IA Key Lite Iniciada. Modelos: {}.", model_info);

    // Configuração da Tray
    let event_loop = EventLoopBuilder::new().build();

    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Sair", true, None);
    let suspend_i = MenuItem::new("Suspender", true, None);
    let start_i = MenuItem::new("Iniciar", true, None);

    let _ = tray_menu.append_items(&[
        &start_i,
        &suspend_i,
        &PredefinedMenuItem::separator(),
        &quit_i,
    ]);

    let icon = if std::path::Path::new("icon.jpg").exists() {
        load_icon(std::path::Path::new("icon.jpg"))
    } else {
        panic!("icon.jpg not found!");
    };

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("jKey Lite")
        .with_icon(icon)
        .build()
        .unwrap();

    // Estado inicial
    suspended.store(false, Ordering::SeqCst);
    info!("Aplicação iniciada.");

    event_loop.run(move |_event, _, control_flow| {
        // Usamos Poll para garantir que o canal de eventos da tray seja verificado constantemente.
        // Em uma aplicação sem janelas, o modo Wait pode não acordar para eventos da tray no Windows.
        *control_flow = ControlFlow::Poll;

        // Processa todos os eventos pendentes do menu da tray
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == quit_i.id() {
                info!("Saindo da aplicação...");
                *control_flow = ControlFlow::Exit;
                // Forçamos a saída com código 0 para evitar o erro de STATUS_CONTROL_C_EXIT
                std::process::exit(0);
            } else if event.id == suspend_i.id() {
                suspended.store(true, Ordering::SeqCst);
                info!("Aplicação suspensa.");
            } else if event.id == start_i.id() {
                suspended.store(false, Ordering::SeqCst);
                info!("Aplicação iniciada.");
            }
        }
    });
}
