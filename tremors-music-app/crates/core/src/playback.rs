// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

#[derive(Debug)]
pub enum AudioCommand {
    Load {
        song_id: i64,
        path: PathBuf,
        generation: u64,
        start: Duration,
        gain: f32,
    },
    Pause,
    Resume,
    Seek(Duration),
    Volume(f32),
    Stop,
}

#[derive(Debug)]
pub enum AudioEvent {
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
    use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
    let mut device = None;
    let mut player: Option<Player> = None;
    let mut generation = 0;
    let mut volume = 0.7;
    let mut awaiting_end = false;
    let mut gain = 1.;
    let (device_tx, device_rx) = mpsc::channel::<String>();
    loop {
        while let Ok(message) = device_rx.try_recv() {
            if let Some(p) = player.take() {
                p.stop();
            }
            device = None;
            awaiting_end = false;
            let _ = tx.send(AudioEvent::Error {
                generation,
                message: format!("Audio device error: {message}. Press Play to reconnect."),
                fatal: true,
            });
        }
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(AudioCommand::Load {
                song_id,
                path,
                generation: next,
                start,
                gain: track_gain,
            }) => {
                generation = next;
                gain = if track_gain.is_finite() {
                    track_gain.clamp(0., 4.)
                } else {
                    1.
                };
                awaiting_end = false;
                if let Some(p) = player.take() {
                    p.stop();
                }
                let result = (|| -> anyhow::Result<_> {
                    if device.is_none() {
                        let errors = device_tx.clone();
                        device = Some(
                            DeviceSinkBuilder::from_default_device()?
                                .with_error_callback(move |error| {
                                    let _ = errors.send(error.to_string());
                                })
                                .open_sink_or_fallback()?,
                        );
                    }
                    let source = Decoder::try_from(std::fs::File::open(&path)?)?;
                    let duration = source.total_duration();
                    let p = Player::connect_new(device.as_ref().unwrap().mixer());
                    p.pause();
                    p.set_volume(volume * gain);
                    p.append(source);
                    if start > Duration::ZERO {
                        p.try_seek(start)?;
                    }
                    p.play();
                    Ok((p, duration))
                })();
                match result {
                    Ok((p, duration)) => {
                        player = Some(p);
                        awaiting_end = true;
                        if tx
                            .send(AudioEvent::Loaded {
                                song_id,
                                generation,
                                duration,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(AudioEvent::Error {
                            generation,
                            message: format!("Cannot play {}: {e:#}", path.display()),
                            fatal: true,
                        });
                    }
                }
            }
            Ok(AudioCommand::Pause) => {
                if let Some(p) = &player {
                    p.pause();
                }
            }
            Ok(AudioCommand::Resume) => {
                if let Some(p) = &player {
                    p.play();
                }
            }
            Ok(AudioCommand::Volume(v)) => {
                volume = if v.is_finite() { v.clamp(0., 1.) } else { 0.7 };
                if let Some(p) = &player {
                    p.set_volume(volume * gain);
                }
            }
            Ok(AudioCommand::Seek(position)) => {
                if let Some(p) = &player
                    && let Err(e) = p.try_seek(position)
                {
                    let _ = tx.send(AudioEvent::Error {
                        generation,
                        message: format!("Seek failed: {e}"),
                        fatal: false,
                    });
                }
            }
            Ok(AudioCommand::Stop) => {
                if let Some(p) = player.take() {
                    p.stop();
                }
                awaiting_end = false;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if let Some(p) = &player {
            if awaiting_end && p.empty() {
                awaiting_end = false;
                if tx.send(AudioEvent::Finished { generation }).is_err() {
                    break;
                }
            } else if tx
                .send(AudioEvent::Position {
                    generation,
                    position: p.get_pos(),
                    paused: p.is_paused(),
                })
                .is_err()
            {
                break;
            }
        }
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
fn worker(rx: Receiver<AudioCommand>, tx: Sender<AudioEvent>) {
    while let Ok(command) = rx.recv() {
        if let AudioCommand::Load { generation, .. } = command {
            let _ = tx.send(AudioEvent::Error {
                generation,
                message: "Native audio output currently targets Windows".into(),
                fatal: true,
            });
        }
    }
}

/// Track ReplayGain with peak limiting; invalid/missing tags leave the gain unchanged.
pub fn track_gain(details: &crate::model::TrackDetails, enabled: bool) -> f32 {
    if !enabled {
        return 1.;
    }
    let Some(db) = details
        .tags
        .get("ReplayGain track")
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|v| v.is_finite())
    else {
        return 1.;
    };
    let mut gain = 10_f32.powf(db.clamp(-60., 12.) / 20.).min(4.);
    if let Some(peak) = details
        .tags
        .get("ReplayGain peak")
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|v| v.is_finite() && *v > 0.)
    {
        gain = gain.min(1. / peak);
    }
    gain
}
