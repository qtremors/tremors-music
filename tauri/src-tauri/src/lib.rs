#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // In production, spawn the backend sidecar
            #[cfg(not(debug_assertions))]
            {
                use tauri_plugin_shell::ShellExt;
                use tauri::Manager;
                
                let shell = app.shell();
                let sidecar = shell.sidecar("tremorsmusic")
                    .expect("failed to create sidecar command");
                
                let (mut _rx, child) = sidecar.spawn()
                    .expect("failed to spawn backend sidecar");
                
                // Store the child handle to kill it on exit
                app.manage(BackendProcess(std::sync::Mutex::new(Some(child))));
                
                log::info!("Backend sidecar started successfully");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // Backend will be killed when the child handle is dropped
                // or we can explicitly kill it if needed across all windows
                 #[cfg(not(debug_assertions))]
                 {
                     // Logic to ensure backend cleanup if needed
                 }
                log::info!("Window destroyed");
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Wrapper to hold the sidecar child process
#[cfg(not(debug_assertions))]
struct BackendProcess(std::sync::Mutex<Option<tauri_plugin_shell::process::CommandChild>>);
