#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod claude_config;
mod server;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_shell::ShellExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tauri::async_runtime::spawn(async move {
                server::start().await;
            });

            build_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error iniciando Lexiius Connector");
}

fn build_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "● Activo — Puerto 47821", false, None::<&str>)?;
    let sep1 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let open = MenuItem::with_id(app, "open", "Abrir Lexiius...", true, None::<&str>)?;
    let sep2 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&status, &sep1, &open, &sep2, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Lexiius Connector")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                let _ = app.shell().open("https://app.lexiius.com", None);
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
                let app = tray.app_handle();
                let _ = app.shell().open("https://app.lexiius.com", None);
            }
        })
        .build(app)?;

    Ok(())
}
