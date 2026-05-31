// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod claude_config;
mod server;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Ocultar de la barra de tareas — solo vive en el system tray
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Iniciar servidor HTTP en background
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                server::start().await;
            });

            // Construir ícono del tray
            build_tray(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error iniciando Lexiius Connector");
}

fn build_tray<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let open_lexiius = MenuItem::with_id(app, "open_lexiius", "Abrir Lexiius...", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let status = MenuItem::with_id(app, "status", "● Activo — Puerto 47821", false, None::<&str>)?;
    let separator2 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &status,
        &separator,
        &open_lexiius,
        &separator2,
        &quit,
    ])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Lexiius Connector")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open_lexiius" => {
                let _ = tauri_plugin_shell::open(
                    &app.shell(),
                    "https://app.lexiius.com",
                    None,
                );
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // Click izquierdo abre Lexiius en el browser
                let app = tray.app_handle();
                let _ = tauri_plugin_shell::open(
                    &app.shell(),
                    "https://app.lexiius.com",
                    None,
                );
            }
        })
        .build(app)?;

    Ok(())
}
