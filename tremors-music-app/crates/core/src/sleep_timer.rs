// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct SleepTimer {
    deadline: Instant,
    pub finish_track: bool,
    fade: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimerState {
    Running { remaining: Duration, gain: f32 },
    Finishing { gain: f32 },
    Expired,
}

impl SleepTimer {
    pub fn new(now: Instant, duration: Duration, finish_track: bool, fade: Duration) -> Self {
        Self {
            deadline: now + duration,
            finish_track,
            fade: fade.min(duration),
        }
    }

    pub fn state(
        &self,
        now: Instant,
        playing: bool,
        track_remaining: Option<Duration>,
    ) -> TimerState {
        let remaining = self.deadline.saturating_duration_since(now);
        let gain = |time: Duration| {
            if self.fade.is_zero() {
                1.
            } else {
                (time.as_secs_f32() / self.fade.as_secs_f32()).clamp(0., 1.)
            }
        };
        if remaining.is_zero() {
            if self.finish_track && playing {
                TimerState::Finishing {
                    gain: track_remaining.map_or(1., gain),
                }
            } else {
                TimerState::Expired
            }
        } else {
            TimerState::Running {
                remaining,
                gain: if self.finish_track {
                    1.
                } else {
                    gain(remaining)
                },
            }
        }
    }

    pub fn stop_at_track_end(&self, now: Instant) -> bool {
        now >= self.deadline
    }
}
