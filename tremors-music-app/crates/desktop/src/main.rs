// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(any(windows, target_os = "linux"))]
mod media;
#[cfg(any(windows, target_os = "linux"))]
mod startup;
#[cfg(any(windows, target_os = "linux"))]
mod ui;

#[cfg(any(windows, target_os = "linux"))]
fn main() {
    let log_path = startup::init();
    match std::panic::catch_unwind(ui::run) {
        Ok(Ok(())) => log::info!("Application closed normally"),
        Ok(Err(error)) => {
            startup::show_error(&format!("{error:#}"), log_path.as_deref());
            std::process::exit(1);
        }
        Err(_) => {
            // The panic hook records the original failure and displays it.
            std::process::exit(1);
        }
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
fn main() {
    eprintln!(
        "Tremors Music currently targets Windows 10/11 and Linux. Run core tests with cargo test -p tremors-core."
    );
    std::process::exit(1);
}
