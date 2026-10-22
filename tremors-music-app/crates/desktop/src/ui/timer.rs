// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Sleep timer orchestration; the clock and fade calculation live in core.
use super::*;
use tremors_core::sleep_timer::{SleepTimer, TimerState};
impl MusicApp {
    pub(in crate::ui) fn start_timer(
        &mut self,
        minutes: u64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sleep_timer = Some(SleepTimer::new(
            Instant::now(),
            Duration::from_secs(minutes * 60),
            self.timer_finish_track,
            Duration::from_secs(if self.timer_fade { 10 } else { 0 }),
        ));
        self.close_overlay(window, cx);
        self.status = format!("Sleep timer set for {minutes} minutes.");
        cx.notify();
    }
    pub(in crate::ui) fn cancel_timer(&mut self) {
        self.sleep_timer = None;
        self.timer_label.clear();
        self.timer_gain = 1.;
        self.send_audio(AudioCommand::Fade(1.));
    }
    pub(in crate::ui) fn expire_timer(&mut self) {
        self.send_audio(AudioCommand::Pause);
        self.is_playing = false;
        self.cancel_timer();
        self.persist_queue();
        self.status = "Sleep timer finished. Playback paused.".into();
    }
    pub(in crate::ui) fn poll_timer(&mut self) -> bool {
        let Some(timer) = &self.sleep_timer else {
            return false;
        };
        let state = timer.state(
            Instant::now(),
            self.is_playing || self.loading,
            Some(Duration::from_secs_f64(
                (self.duration - self.position).max(0.),
            )),
        );
        let (label, gain) = match state {
            TimerState::Running { remaining, gain } => {
                let seconds = remaining.as_secs().saturating_add(1);
                (format!("{}:{:02}", seconds / 60, seconds % 60), gain)
            }
            TimerState::Finishing { gain } => ("After this track".into(), gain),
            TimerState::Expired => {
                self.expire_timer();
                return true;
            }
        };
        let changed = self.timer_label != label;
        self.timer_label = label;
        if (self.timer_gain - gain).abs() > 0.005 {
            self.timer_gain = gain;
            self.send_audio(AudioCommand::Fade(gain));
        }
        changed
    }
}
