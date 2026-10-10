// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use log::{LevelFilter, Log, Metadata, Record};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

struct FileLogger(Mutex<File>);

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata())
            && let Ok(mut file) = self.0.lock()
        {
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let _ = writeln!(
                file,
                "{time} {} {}: {}",
                record.level(),
                record.target(),
                record.args()
            );
            let _ = file.flush();
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.0.lock() {
            let _ = file.flush();
        }
    }
}

pub fn init() -> Option<PathBuf> {
    let preferred = tremors_core::settings::AppPaths::discover()
        .ok()
        .map(|paths| paths.data.join("startup.log"));
    let fallback = std::env::temp_dir().join("TremorsMusic-startup.log");
    let log_path = preferred.into_iter().chain([fallback]).find_map(|path| {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok()
            .map(|file| (path, file))
    });
    let path = log_path.and_then(|(path, file)| {
        log::set_boxed_logger(Box::new(FileLogger(Mutex::new(file))))
            .ok()
            .map(|()| path)
    });
    log::set_max_level(LevelFilter::Info);
    log::info!(
        "Tremors Music {} starting on {} ({})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        if cfg!(debug_assertions) {
            "development"
        } else {
            "release"
        }
    );
    if let Some(path) = &path {
        log::info!("Startup log: {}", path.display());
    }
    let panic_path = path.clone();
    std::panic::set_hook(Box::new(move |info| {
        show_error(
            &format!("Unexpected startup or runtime failure:\n{info}"),
            panic_path.as_deref(),
        );
    }));
    path
}

pub fn show_error(error: &str, log_path: Option<&Path>) {
    log::error!("{error}");
    log::logger().flush();
    eprintln!("Tremors Music could not start: {error}");
    let description = match log_path {
        Some(path) => format!("{error}\n\nDetails saved to:\n{}", path.display()),
        None => error.to_owned(),
    };
    rfd::MessageDialog::new()
        .set_title("Tremors Music could not start")
        .set_description(description)
        .set_level(rfd::MessageLevel::Error)
        .show();
}
