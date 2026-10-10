// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use crate::{
    library::Library,
    model::Snapshot,
    queue::QueueState,
    scanner::{self, ScanProgress},
    settings::{AppPaths, Preferences},
};
use anyhow::Result;
use notify::{RecursiveMode, Watcher};
use std::time::Duration;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
    },
};

#[derive(Debug)]
pub enum LibraryCommand {
    Refresh,
    Session(QueueState),
    ReplaceFolder(PathBuf, PathBuf),
    Reset(bool),
    CleanArtwork,
    AddFolder(PathBuf),
    AddFolderAndScan(PathBuf),
    RelocateAndScan(PathBuf, PathBuf),
    RemoveFolder(PathBuf),
    Scan,
    Favorite(i64),
    Played(i64),
    Preferences(Preferences),
    CreatePlaylist(String),
    RenamePlaylist(i64, String),
    DeletePlaylist(i64),
    AddToPlaylist(i64, i64),
    RemoveEntry(i64),
    MoveEntry(i64, i64, isize),
}

pub enum LibraryEvent {
    Snapshot(Arc<Snapshot>),
    ScanStarted,
    ScanProgress(ScanProgress),
    ScanFinished {
        message: String,
        warnings: Vec<String>,
    },
    Error(String),
    Warning(String),
}

pub struct LibraryHandle {
    pub commands: Sender<LibraryCommand>,
    pub events: Receiver<LibraryEvent>,
    pub cancel: Arc<AtomicBool>,
}

pub fn spawn(paths: AppPaths) -> LibraryHandle {
    let (commands, rx) = mpsc::channel();
    let (tx, events) = mpsc::channel();
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = cancel.clone();
    std::thread::Builder::new()
        .name("tremors-library".into())
        .spawn(move || {
            if let Err(e) = worker(paths, rx, &tx, &worker_cancel) {
                let _ = tx.send(LibraryEvent::Error(format!("Library error: {e:#}")));
            }
        })
        .expect("Could not start library worker");
    LibraryHandle {
        commands,
        events,
        cancel,
    }
}

fn worker(
    paths: AppPaths,
    rx: Receiver<LibraryCommand>,
    tx: &Sender<LibraryEvent>,
    cancel: &AtomicBool,
) -> Result<()> {
    let mut library = Library::open(&paths.database)?;
    tx.send(LibraryEvent::Snapshot(Arc::new(library.snapshot()?)))?;
    let dirty = Arc::new(AtomicU64::new(0));
    let changed = dirty.clone();
    let watch_errors = tx.clone();
    let mut watcher =
        notify::recommended_watcher(move |event: notify::Result<notify::Event>| match event {
            Ok(event) if !matches!(event.kind, notify::EventKind::Access(_)) => {
                changed.store(clock_ms(), Ordering::Relaxed)
            }
            Err(e) => {
                let _ = watch_errors.send(LibraryEvent::Warning(format!("Folder watcher: {e}")));
            }
            _ => {}
        })
        .ok();
    let mut watched = Vec::new();
    let mut last_scan = 0;
    loop {
        let (roots, watch_enabled) = library.watch_config()?;
        if let Some(watcher) = &mut watcher {
            let wanted = if watch_enabled {
                roots.clone()
            } else {
                Vec::new()
            };
            for root in watched.iter().filter(|r| !wanted.contains(r)) {
                let _ = watcher.unwatch(root);
            }
            watched.retain(|r| wanted.contains(r));
            for root in wanted {
                if !watched.contains(&root)
                    && root.is_dir()
                    && watcher.watch(&root, RecursiveMode::Recursive).is_ok()
                {
                    watched.push(root);
                }
            }
        }
        let mut command = match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(command) => command,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let change = dirty.load(Ordering::Relaxed);
                if watch_enabled
                    && change > 0
                    && clock_ms().saturating_sub(change) >= 1500
                    && clock_ms().saturating_sub(last_scan) >= 5000
                    && !roots.is_empty()
                {
                    dirty.store(0, Ordering::Relaxed);
                    cancel.store(false, Ordering::Relaxed);
                    LibraryCommand::Scan
                } else {
                    continue;
                }
            }
        };
        let folder_change = match &command {
            LibraryCommand::AddFolderAndScan(path) => Some(library.add_root(path)),
            LibraryCommand::RelocateAndScan(old, new) => Some(library.replace_root(old, new)),
            _ => None,
        };
        if let Some(result) = folder_change {
            if let Err(e) = result {
                tx.send(LibraryEvent::Error(format!("{e:#}")))?;
                continue;
            }
            tx.send(LibraryEvent::Snapshot(Arc::new(library.snapshot()?)))?;
            command = LibraryCommand::Scan;
        }
        if let LibraryCommand::Session(state) = &command {
            if let Err(e) = library.save_session(state) {
                tx.send(LibraryEvent::Error(format!(
                    "Cannot save playback state: {e:#}"
                )))?;
            }
            continue;
        }
        let result = (|| -> Result<()> {
            match command {
                LibraryCommand::Refresh => {}
                LibraryCommand::Session(_)
                | LibraryCommand::AddFolderAndScan(_)
                | LibraryCommand::RelocateAndScan(_, _) => unreachable!(),
                LibraryCommand::ReplaceFolder(old, new) => library.replace_root(&old, &new)?,
                LibraryCommand::Reset(remove_folders) => library.reset(remove_folders)?,
                LibraryCommand::CleanArtwork => {
                    scanner::cleanup_covers(&paths.covers, &library.snapshot()?.songs)?;
                }
                LibraryCommand::AddFolder(path) => library.add_root(&path)?,
                LibraryCommand::RemoveFolder(path) => library.remove_root(&path)?,
                LibraryCommand::Favorite(id) => library.favorite(id)?,
                LibraryCommand::Played(id) => library.played(id)?,
                LibraryCommand::Preferences(p) => library.save_preferences(&p)?,
                LibraryCommand::CreatePlaylist(name) => {
                    library.create_playlist(&name)?;
                }
                LibraryCommand::RenamePlaylist(id, name) => library.rename_playlist(id, &name)?,
                LibraryCommand::DeletePlaylist(id) => library.delete_playlist(id)?,
                LibraryCommand::AddToPlaylist(pl, song) => library.add_to_playlist(pl, song)?,
                LibraryCommand::RemoveEntry(id) => library.remove_entry(id)?,
                LibraryCommand::MoveEntry(pl, id, delta) => library.move_entry(pl, id, delta)?,
                LibraryCommand::Scan => {
                    last_scan = clock_ms();
                    dirty.store(0, Ordering::Relaxed);
                    let snapshot = library.snapshot()?;
                    let roots = snapshot.roots;
                    if roots.is_empty() {
                        anyhow::bail!("Add a music folder before scanning");
                    }
                    tx.send(LibraryEvent::ScanStarted)?;
                    let mut imported = 0;
                    let mut warnings = Vec::new();
                    let mut cancelled = false;
                    for root in roots {
                        if cancel.load(Ordering::Relaxed) {
                            cancelled = true;
                            break;
                        }
                        match scanner::scan_cached(
                            &root,
                            &paths.covers,
                            cancel,
                            &snapshot.songs,
                            |p| {
                                let _ = tx.send(LibraryEvent::ScanProgress(p));
                            },
                        ) {
                            Ok(scan) => {
                                warnings.extend(scan.warnings);
                                if scan.cancelled {
                                    cancelled = true;
                                    break;
                                }
                                imported += scan.songs.len();
                                library.apply_scan(
                                    &root,
                                    &scan.songs,
                                    &scan.seen,
                                    scan.can_prune,
                                )?;
                            }
                            Err(e) => warnings.push(format!("{}: {e:#}", root.display())),
                        }
                    }
                    tx.send(LibraryEvent::ScanFinished {
                        message: if cancelled {
                            "Scan cancelled. Completed folders were saved.".into()
                        } else {
                            format!(
                                "Scan complete: {imported} tracks read, {} warnings",
                                warnings.len()
                            )
                        },
                        warnings,
                    })?;
                }
            }
            Ok(())
        })();
        if let Err(e) = result {
            tx.send(LibraryEvent::Error(format!("{e:#}")))?;
        }
        tx.send(LibraryEvent::Snapshot(Arc::new(library.snapshot()?)))?;
    }
    Ok(())
}

fn clock_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
