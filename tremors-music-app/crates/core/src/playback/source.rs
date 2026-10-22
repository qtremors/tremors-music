// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Predecoded sources share a Rodio queue. Track boundaries never wait for the UI loop.
use super::{
    NextTrack,
    effects::{Effects, Processor},
};
use rodio::{
    Source,
    source::{SeekError, TrackPosition},
};
use std::{collections::HashMap, path::PathBuf};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc::Sender,
    },
    time::Duration,
};
#[derive(Default)]
pub(super) struct Progress {
    pub token: AtomicU64,
    pub micros: AtomicU64,
}
pub(super) struct Started {
    pub generation: u64,
    pub track: NextTrack,
    pub duration: Option<Duration>,
}
pub(super) type OpenFiles = Arc<Mutex<HashMap<PathBuf, usize>>>;

struct FileLease {
    path: PathBuf,
    files: OpenFiles,
}
impl FileLease {
    fn new(path: PathBuf, files: OpenFiles) -> Self {
        *files.lock().unwrap().entry(path.clone()).or_default() += 1;
        Self { path, files }
    }
}
impl Drop for FileLease {
    fn drop(&mut self) {
        let mut files = self.files.lock().unwrap();
        if let Some(count) = files.get_mut(&self.path) {
            *count -= 1;
            if *count == 0 {
                files.remove(&self.path);
            }
        }
    }
}

pub(super) struct Shared {
    pub open_files: OpenFiles,
    pub generation: u64,
    pub progress: Arc<Progress>,
    pub effects: Arc<Mutex<Effects>>,
    pub started: Sender<Started>,
}
pub(super) struct TrackSource<S: Source> {
    source: TrackPosition<S>,
    track: Option<NextTrack>,
    token: u64,
    duration: Option<Duration>,
    shared: Shared,
    processor: Processor,
    samples: u32,
    effects: Effects,
    // Declared last: the decoder must drop before its file lease is released.
    _file_lease: FileLease,
}
impl<S: Source> TrackSource<S> {
    pub fn new(source: S, track: NextTrack, shared: Shared) -> Self {
        let effects = shared.effects.lock().unwrap().clone();
        let processor = Processor::new(
            source.channels().get() as usize,
            source.sample_rate().get(),
            effects.clone(),
        );
        let duration = source.total_duration();
        let token = track.token;
        let file_lease = FileLease::new(track.path.clone(), shared.open_files.clone());
        Self {
            source: source.track_position(),
            track: Some(track),
            token,
            duration,
            shared,
            processor,
            samples: 0,
            effects,
            _file_lease: file_lease,
        }
    }
}
impl<S: Source> Iterator for TrackSource<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let sample = self.source.next()?;
        if let Some(track) = self.track.take() {
            self.shared.progress.micros.store(
                self.source.get_pos().as_micros().min(u64::MAX as u128) as u64,
                Ordering::Relaxed,
            );
            self.shared
                .progress
                .token
                .store(self.token, Ordering::Release);
            let _ = self.shared.started.send(Started {
                generation: self.shared.generation,
                track,
                duration: self.duration,
            });
        }
        self.samples += 1;
        if self.samples >= 1024 {
            self.samples = 0;
            self.shared.progress.micros.store(
                self.source.get_pos().as_micros().min(u64::MAX as u128) as u64,
                Ordering::Relaxed,
            );
            if let Ok(effects) = self.shared.effects.try_lock()
                && *effects != self.effects
            {
                self.effects = effects.clone();
                self.processor.update(self.effects.clone());
            }
        }
        Some(self.processor.sample(sample))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.source.size_hint()
    }
}
impl<S: Source> Source for TrackSource<S> {
    fn current_span_len(&self) -> Option<usize> {
        self.source.current_span_len()
    }
    fn channels(&self) -> rodio::ChannelCount {
        self.source.channels()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        self.source.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.source.try_seek(pos)?;
        self.processor.reset();
        self.shared.progress.micros.store(
            pos.as_micros().min(u64::MAX as u128) as u64,
            Ordering::Relaxed,
        );
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;
    #[test]
    fn preloaded_sources_join_without_silence_and_report_exact_boundary() {
        let (queue, mut output) = rodio::queue::queue(false);
        let (started, rx) = std::sync::mpsc::channel();
        let progress = Arc::new(Progress::default());
        let effects = Arc::new(Mutex::new(Effects::default()));
        for (token, value) in [(1, 0.25), (2, 0.5)] {
            queue.append(TrackSource::new(
                SamplesBuffer::new(
                    2.try_into().unwrap(),
                    48000.try_into().unwrap(),
                    vec![value; 480],
                ),
                NextTrack {
                    token,
                    song_id: token as i64,
                    path: Default::default(),
                    gain: 1.,
                },
                Shared {
                    open_files: Default::default(),
                    generation: 1,
                    progress: progress.clone(),
                    effects: effects.clone(),
                    started: started.clone(),
                },
            ));
        }
        let actual: Vec<_> = output.by_ref().collect();
        assert_eq!(actual.len(), 960);
        assert!(actual[..480].iter().all(|s| *s == 0.25));
        assert!(actual[480..].iter().all(|s| *s == 0.5));
        assert_eq!(
            rx.try_iter()
                .map(|event| event.track.token)
                .collect::<Vec<_>>(),
            [1, 2]
        );
    }
    #[test]
    fn replacing_preload_never_consumes_stale_track() {
        let (queue, mut output) = rodio::queue::queue(false);
        let (started, rx) = std::sync::mpsc::channel();
        let progress = Arc::new(Progress::default());
        let effects = Arc::new(Mutex::new(Effects::default()));
        let source = |token, value| {
            TrackSource::new(
                SamplesBuffer::new(
                    1.try_into().unwrap(),
                    8000.try_into().unwrap(),
                    vec![value; 80],
                ),
                NextTrack {
                    token,
                    song_id: token as i64,
                    path: Default::default(),
                    gain: 1.,
                },
                Shared {
                    open_files: Default::default(),
                    generation: 1,
                    progress: progress.clone(),
                    effects: effects.clone(),
                    started: started.clone(),
                },
            )
        };
        queue.append(source(1, 0.25));
        assert_eq!(output.next(), Some(0.25));
        queue.append(source(2, 0.5));
        queue.clear();
        queue.append(source(3, 0.75));
        let rest: Vec<_> = output.collect();
        assert_eq!(rest.len(), 159);
        assert!(rest[..79].iter().all(|s| *s == 0.25));
        assert!(rest[79..].iter().all(|s| *s == 0.75));
        assert_eq!(
            rx.try_iter().map(|e| e.track.token).collect::<Vec<_>>(),
            [1, 3]
        );
    }
    #[test]
    fn file_leases_follow_preloaded_and_active_decoders() {
        let (queue, mut output) = rodio::queue::queue(false);
        let (started, _) = std::sync::mpsc::channel();
        let files: OpenFiles = Default::default();
        let path = PathBuf::from("repeat.mp3");
        for token in 1..=2 {
            queue.append(TrackSource::new(
                SamplesBuffer::new(
                    1.try_into().unwrap(),
                    8000.try_into().unwrap(),
                    vec![0.25; 80],
                ),
                NextTrack {
                    song_id: 1,
                    path: path.clone(),
                    gain: 1.,
                    token,
                },
                Shared {
                    generation: 1,
                    open_files: files.clone(),
                    progress: Default::default(),
                    effects: Arc::new(Mutex::new(Effects::default())),
                    started: started.clone(),
                },
            ));
        }
        assert_eq!(files.lock().unwrap()[&path], 2);
        assert_eq!(output.next(), Some(0.25));
        queue.clear();
        assert_eq!(files.lock().unwrap()[&path], 1);
        drop(output);
        assert!(!files.lock().unwrap().contains_key(&path));
    }
}
