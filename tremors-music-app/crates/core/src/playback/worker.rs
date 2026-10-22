// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Device ownership and command ordering. Decoding and preloading run off the UI thread.
use super::source::{Progress, Shared, Started, TrackSource};
use super::*;
use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use std::sync::{Arc, Mutex, atomic::Ordering};
pub(super) fn run(rx: Receiver<AudioCommand>, tx: Sender<AudioEvent>) {
    let mut device = None;
    let mut player: Option<Player> = None;
    let mut queue: Option<Arc<rodio::queue::SourcesQueueInput>> = None;
    let mut generation = 0;
    let mut volume = 0.7;
    let mut fade = 1.;
    let mut speed = 1.;
    let mut awaiting_end = false;
    let mut active_token = 0;
    let effects = Arc::new(Mutex::new(effects::Effects::default()));
    let open_files: super::source::OpenFiles = Default::default();
    let mut progress = Arc::new(Progress::default());
    let (started_tx, started_rx) = mpsc::channel::<Started>();
    let (device_tx, device_rx) = mpsc::channel::<String>();
    loop {
        while let Ok(event) = started_rx.try_recv() {
            if event.generation == generation && awaiting_end {
                active_token = event.track.token;
                if active_token != 0 {
                    let _ = tx.send(AudioEvent::Advanced {
                        generation,
                        track: event.track,
                        duration: event.duration,
                    });
                }
            }
        }
        while let Ok(message) = device_rx.try_recv() {
            if let Some(p) = player.take() {
                p.stop();
            }
            queue = None;
            device = None;
            awaiting_end = false;
            let _ = tx.send(AudioEvent::Error {
                generation,
                message: format!("Audio device error: {message}. Press Play to reconnect."),
                fatal: true,
            });
        }
        match rx.recv_timeout(Duration::from_millis(30)) {
            Ok(AudioCommand::Load {
                song_id,
                path,
                generation: next,
                start,
                gain,
            }) => {
                generation = next;
                awaiting_end = false;
                active_token = 0;
                queue = None;
                if let Some(p) = player.take() {
                    p.stop();
                }
                progress = Arc::new(Progress::default());
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
                    let decoder = Decoder::try_from(std::fs::File::open(&path)?)?;
                    let duration = decoder.total_duration();
                    let mut source = TrackSource::new(
                        decoder.amplify(safe_gain(gain)),
                        NextTrack {
                            song_id,
                            path: path.clone(),
                            gain,
                            token: 0,
                        },
                        Shared {
                            open_files: open_files.clone(),
                            generation,
                            progress: progress.clone(),
                            effects: effects.clone(),
                            started: started_tx.clone(),
                        },
                    );
                    if start > Duration::ZERO {
                        source.try_seek(start)?;
                    }
                    let (input, output) = rodio::queue::queue(false);
                    input.append(source);
                    let p = Player::connect_new(device.as_ref().unwrap().mixer());
                    p.pause();
                    p.set_volume(volume * fade);
                    p.set_speed(speed);
                    p.append(output);
                    p.play();
                    Ok((p, input, duration))
                })();
                match result {
                    Ok((p, input, duration)) => {
                        player = Some(p);
                        queue = Some(input);
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
            Ok(AudioCommand::PrepareNext {
                generation: expected,
                track,
            }) if expected == generation => {
                if let Some(input) = &queue {
                    input.clear();
                    if let Some(track) = track
                        && awaiting_end
                    {
                        match std::fs::File::open(&track.path)
                            .map_err(anyhow::Error::from)
                            .and_then(|file| Decoder::try_from(file).map_err(Into::into))
                        {
                            Ok(decoder) => input.append(TrackSource::new(
                                decoder.amplify(safe_gain(track.gain)),
                                track,
                                Shared {
                                    open_files: open_files.clone(),
                                    generation,
                                    progress: progress.clone(),
                                    effects: effects.clone(),
                                    started: started_tx.clone(),
                                },
                            )),
                            Err(e) => {
                                let _ = tx.send(AudioEvent::Error {
                                    generation,
                                    message: format!("Next track could not be preloaded: {e:#}"),
                                    fatal: false,
                                });
                            }
                        }
                    }
                }
            }
            Ok(AudioCommand::PrepareNext { .. }) => {}
            Ok(AudioCommand::Effects(value)) => {
                *effects.lock().unwrap() = value.sanitized();
            }
            Ok(AudioCommand::Speed(value)) => {
                speed = if value.is_finite() {
                    value.clamp(0.5, 2.)
                } else {
                    1.
                };
                if let Some(p) = &player {
                    p.set_speed(speed);
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
            Ok(AudioCommand::Volume(value)) => {
                volume = if value.is_finite() {
                    value.clamp(0., 1.)
                } else {
                    0.7
                };
                if let Some(p) = &player {
                    p.set_volume(volume * fade);
                }
            }
            Ok(AudioCommand::Fade(value)) => {
                fade = if value.is_finite() {
                    value.clamp(0., 1.)
                } else {
                    1.
                };
                if let Some(p) = &player {
                    p.set_volume(volume * fade);
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
            Ok(AudioCommand::ReleaseForEdit { path, ack }) => {
                // Sources own leases until their decoders drop, including a source currently
                // crossing from the preload queue into the mixer on the audio thread.
                let open = open_files.lock().unwrap().contains_key(&path);
                if open {
                    if let Some(p) = player.take() {
                        p.stop();
                    }
                    queue = None;
                    device = None;
                    awaiting_end = false;
                }
                let _ = tx.send(AudioEvent::EditReady {
                    generation,
                    released: open,
                });
                let _ = ack.send(());
            }
            Ok(AudioCommand::Release(ack)) => {
                if let Some(p) = player.take() {
                    p.stop();
                }
                queue = None;
                device = None;
                awaiting_end = false;
                let _ = ack.send(());
            }
            Ok(AudioCommand::Stop) => {
                if let Some(p) = player.take() {
                    p.stop();
                }
                queue = None;
                awaiting_end = false;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if let Some(p) = &player {
            // Drain boundary events before position/end, including boundaries consumed during a command.
            while let Ok(event) = started_rx.try_recv() {
                if event.generation == generation && awaiting_end {
                    active_token = event.track.token;
                    if active_token != 0 {
                        let _ = tx.send(AudioEvent::Advanced {
                            generation,
                            track: event.track,
                            duration: event.duration,
                        });
                    }
                }
            }
            if awaiting_end && p.empty() {
                awaiting_end = false;
                if tx.send(AudioEvent::Finished { generation }).is_err() {
                    break;
                }
            } else if progress.token.load(Ordering::Acquire) == active_token
                && tx
                    .send(AudioEvent::Position {
                        generation,
                        position: Duration::from_micros(progress.micros.load(Ordering::Relaxed)),
                        paused: p.is_paused(),
                    })
                    .is_err()
            {
                break;
            }
        }
    }
}
fn safe_gain(gain: f32) -> f32 {
    if gain.is_finite() {
        gain.clamp(0., 4.)
    } else {
        1.
    }
}
