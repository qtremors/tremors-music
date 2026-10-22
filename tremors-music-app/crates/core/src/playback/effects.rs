// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Stateful audio processing, independent of the UI and output device.
use serde::{Deserialize, Serialize};
pub const FREQUENCIES: [f32; 10] = [
    31., 62., 125., 250., 500., 1000., 2000., 4000., 8000., 16000.,
];
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Effects {
    pub enabled: bool,
    pub bands: [f32; 10],
    pub preamp_db: f32,
    pub balance: f32,
}
impl Default for Effects {
    fn default() -> Self {
        Self {
            enabled: false,
            bands: [0.; 10],
            preamp_db: 0.,
            balance: 0.,
        }
    }
}
impl Effects {
    pub fn sanitized(mut self) -> Self {
        for gain in &mut self.bands {
            *gain = finite(*gain, 0.).clamp(-12., 12.);
        }
        self.preamp_db = finite(self.preamp_db, 0.).clamp(-24., 12.);
        self.balance = finite(self.balance, 0.).clamp(-1., 1.);
        self
    }
}
fn finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}
#[derive(Clone, Copy, Default)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}
impl Biquad {
    fn peak(frequency: f32, gain: f32, rate: f32) -> Self {
        if frequency >= rate * 0.49 || gain.abs() < 0.001 {
            return Self {
                b0: 1.,
                ..Self::default()
            };
        }
        let a = 10_f32.powf(gain / 40.);
        let w = 2. * std::f32::consts::PI * frequency / rate;
        let alpha = w.sin() / 2.;
        let a0 = 1. + alpha / a;
        Self {
            b0: (1. + alpha * a) / a0,
            b1: -2. * w.cos() / a0,
            b2: (1. - alpha * a) / a0,
            a1: -2. * w.cos() / a0,
            a2: (1. - alpha / a) / a0,
            ..Self::default()
        }
    }
    fn sample(&mut self, input: f32) -> f32 {
        let out = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * out + self.z2;
        self.z2 = self.b2 * input - self.a2 * out;
        out
    }
}
pub(super) struct Processor {
    filters: Vec<[Biquad; 10]>,
    effects: Effects,
    channel: usize,
    rate: u32,
    preamp: f32,
}
impl Processor {
    pub fn new(channels: usize, rate: u32, effects: Effects) -> Self {
        let mut p = Self {
            filters: vec![[Biquad::default(); 10]; channels],
            effects: Effects::default(),
            channel: 0,
            rate,
            preamp: 1.,
        };
        p.update(effects);
        p
    }
    pub fn update(&mut self, effects: Effects) {
        self.effects = effects.sanitized();
        self.preamp = 10_f32.powf(self.effects.preamp_db / 20.);
        for channel in &mut self.filters {
            for (i, filter) in channel.iter_mut().enumerate() {
                *filter = Biquad::peak(FREQUENCIES[i], self.effects.bands[i], self.rate as f32);
            }
        }
    }
    pub fn reset(&mut self) {
        for channel in &mut self.filters {
            for f in channel {
                f.z1 = 0.;
                f.z2 = 0.;
            }
        }
        self.channel = 0;
    }
    pub fn sample(&mut self, input: f32) -> f32 {
        let channel = self.channel;
        self.channel = (self.channel + 1) % self.filters.len();
        let mut value = input;
        if self.effects.enabled {
            for filter in &mut self.filters[channel] {
                value = filter.sample(value);
            }
        }
        value *= self.preamp;
        if self.filters.len() >= 2 {
            if channel == 0 && self.effects.balance > 0. {
                value *= 1. - self.effects.balance;
            } else if channel == 1 && self.effects.balance < 0. {
                value *= 1. + self.effects.balance;
            }
        }
        if value.is_finite() {
            value.clamp(-1., 1.)
        } else {
            0.
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bypass_balance_and_invalid_values_are_safe() {
        let mut p = Processor::new(2, 48000, Effects::default());
        assert_eq!(p.sample(0.25), 0.25);
        assert_eq!(p.sample(-0.25), -0.25);
        p.update(Effects {
            balance: 1.,
            ..Effects::default()
        });
        assert_eq!(p.sample(0.25), 0.);
        assert_eq!(p.sample(0.25), 0.25);
        p.update(Effects {
            preamp_db: f32::NAN,
            balance: f32::INFINITY,
            ..Effects::default()
        });
        assert_eq!(p.sample(0.25), 0.25);
    }
    #[test]
    fn peak_filter_changes_frequency_energy_without_unstable_output() {
        let mut effects = Effects {
            enabled: true,
            ..Effects::default()
        };
        effects.bands[5] = -12.;
        let mut p = Processor::new(1, 48000, effects);
        let mut input = 0.;
        let mut output = 0.;
        for i in 0..48000 {
            let sample = (i as f32 * 2. * std::f32::consts::PI * 1000. / 48000.).sin() * 0.2;
            let filtered = p.sample(sample);
            if i > 24000 {
                input += sample * sample;
                output += filtered * filtered;
            }
            assert!(filtered.is_finite());
        }
        assert!(output < input * 0.15);
    }
}
