use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Spawn the Python backend sidecar
            let sidecar = app.shell().sidecar("tremorsmusic")
                .expect("Failed to create sidecar command");
            
            let (mut rx, _child) = sidecar.spawn()
                .expect("Failed to spawn backend sidecar");
            
            // Spawn async task to handle sidecar output
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stdout(line) => {
                            let output = String::from_utf8_lossy(&line);
                            log::info!("[Backend] {}", output);
                        }
                        CommandEvent::Stderr(line) => {
                            let output = String::from_utf8_lossy(&line);
                            log::warn!("[Backend] {}", output);
                        }
                        CommandEvent::Error(err) => {
                            log::error!("[Backend Error] {}", err);
                        }
                        CommandEvent::Terminated(payload) => {
                            log::info!("[Backend] Process terminated with code: {:?}", payload.code);
                        }
                        _ => {}
                    }
                }
            });
            
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
