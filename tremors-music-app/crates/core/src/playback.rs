// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

pub mod effects;
#[cfg(any(windows, target_os = "linux"))]
mod source;
#[cfg(any(windows, target_os = "linux"))]
mod worker;

use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

#[derive(Clone, Debug, PartialEq)]
pub struct NextTrack {
    pub song_id: i64,
    pub path: PathBuf,
    pub gain: f32,
    pub token: u64,
}

#[derive(Debug)]
pub enum AudioCommand {
    Load {
        song_id: i64,
        path: PathBuf,
        generation: u64,
        start: Duration,
        gain: f32,
    },
    PrepareNext {
        generation: u64,
        track: Option<NextTrack>,
    },
    Effects(effects::Effects),
    Speed(f32),
    Pause,
    Resume,
    Seek(Duration),
    Volume(f32),
    Stop,
    /// Release the decoder before replacing a file in the metadata worker.
    Release(Sender<()>),
    ReleaseForEdit {
        path: PathBuf,
        ack: Sender<()>,
    },
    Fade(f32),
}

#[derive(Debug)]
pub enum AudioEvent {
    EditReady {
        generation: u64,
        released: bool,
    },
    Advanced {
        generation: u64,
        track: NextTrack,
        duration: Option<Duration>,
    },
    Loaded {
        song_id: i64,
        generation: u64,
        duration: Option<Duration>,
    },
    Position {
        generation: u64,
        position: Duration,
        paused: bool,
    },
    Finished {
        generation: u64,
    },
    Error {
        generation: u64,
        message: String,
        fatal: bool,
    },
}

pub struct AudioHandle {
    pub commands: Sender<AudioCommand>,
    pub events: Receiver<AudioEvent>,
}

pub fn spawn() -> AudioHandle {
    let (commands, rx) = mpsc::channel();
    let (tx, events) = mpsc::channel();
    std::thread::Builder::new()
        .name("tremors-audio".into())
        .spawn(move || worker(rx, tx))
        .expect("Could not start audio worker");
    AudioHandle { commands, events }
}

#[cfg(any(windows, target_os = "linux"))]
fn worker(rx: Receiver<AudioCommand>, tx: Sender<AudioEvent>) {
    worker::run(rx, tx);
}

#[cfg(not(any(windows, target_os = "linux")))]
fn worker(rx: Receiver<AudioCommand>, tx: Sender<AudioEvent>) {
    while let Ok(command) = rx.recv() {
        if let AudioCommand::Release(ack) | AudioCommand::ReleaseForEdit { ack, .. } = &command {
            let _ = ack.send(());
        }
        if let AudioCommand::Load { generation, .. } = command {
            let _ = tx.send(AudioEvent::Error {
                generation,
                message: "Audio output requires Windows or Linux".into(),
                fatal: true,
            });
        }
    }
}

/// Track ReplayGain with peak limiting; invalid/missing tags leave the gain unchanged.
pub fn track_gain(details: &crate::model::TrackDetails, enabled: bool) -> f32 {
    replay_gain(details, enabled, false)
}

pub fn replay_gain(details: &crate::model::TrackDetails, enabled: bool, album: bool) -> f32 {
    if !enabled {
        return 1.;
    }
    let Some(db) = details
        .tags
        .get(if album {
            "ReplayGain album"
        } else {
            "ReplayGain track"
        })
        .or_else(|| details.tags.get("ReplayGain track"))
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|v| v.is_finite())
    else {
        return 1.;
    };
    let mut gain = 10_f32.powf(db.clamp(-60., 12.) / 20.).min(4.);
    if let Some(peak) = details
        .tags
        .get(if album && details.tags.contains_key("ReplayGain album") {
            "ReplayGain album peak"
        } else {
            "ReplayGain peak"
        })
        .or_else(|| details.tags.get("ReplayGain peak"))
        .and_then(|s| s.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v > 0.)
    {
        gain = gain.min(1. / peak);
    }
    gain
}
