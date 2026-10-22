// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Audio preference controls send immutable settings to the playback worker.
use super::*;
use tremors_core::playback::effects::{Effects, FREQUENCIES};
pub(super) struct Controls {
    bands: Vec<Entity<SliderState>>,
    preamp: Entity<SliderState>,
    balance: Entity<SliderState>,
    speed: Entity<SliderState>,
    _subscriptions: Vec<Subscription>,
}
impl Controls {
    pub fn new(window: &mut Window, cx: &mut Context<MusicApp>) -> Self {
        let bands: Vec<_> = (0..10)
            .map(|_| {
                cx.new(|_| {
                    SliderState::new()
                        .min(-12.)
                        .max(12.)
                        .step(0.5)
                        .default_value(0.)
                })
            })
            .collect();
        let preamp = cx.new(|_| {
            SliderState::new()
                .min(-24.)
                .max(12.)
                .step(0.5)
                .default_value(0.)
        });
        let balance = cx.new(|_| {
            SliderState::new()
                .min(-100.)
                .max(100.)
                .step(5.)
                .default_value(0.)
        });
        let speed = cx.new(|_| {
            SliderState::new()
                .min(0.5)
                .max(2.)
                .step(0.05)
                .default_value(1.)
        });
        let mut subscriptions = Vec::new();
        for (index, input) in bands.iter().chain([&preamp, &balance, &speed]).enumerate() {
            subscriptions.push(
                cx.subscribe_in(input, window, move |this, _, event, _, cx| match event {
                    SliderEvent::Change(SliderValue::Single(value)) => {
                        match index {
                            0..10 => this.preferences.audio_effects.bands[index] = *value,
                            10 => this.preferences.audio_effects.preamp_db = *value,
                            11 => this.preferences.audio_effects.balance = *value / 100.,
                            12 => this.preferences.playback_speed = *value,
                            _ => unreachable!(),
                        }
                        this.send_audio(AudioCommand::Effects(
                            this.preferences.audio_effects.clone(),
                        ));
                        this.send_audio(AudioCommand::Speed(this.preferences.playback_speed));
                        cx.notify();
                    }
                    SliderEvent::Release(_) => this.persist_modes(),
                    _ => {}
                }),
            );
        }
        Self {
            bands,
            preamp,
            balance,
            speed,
            _subscriptions: subscriptions,
        }
    }
}
impl MusicApp {
    pub(in crate::ui) fn initialize_sound(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.preferences.audio_effects = self.preferences.audio_effects.clone().sanitized();
        if !self.preferences.playback_speed.is_finite() {
            self.preferences.playback_speed = 1.;
        }
        self.preferences.playback_speed = self.preferences.playback_speed.clamp(0.5, 2.);
        for (i, band) in self.sound.bands.iter().enumerate() {
            band.update(cx, |slider, cx| {
                slider.set_value(self.preferences.audio_effects.bands[i], window, cx)
            });
        }
        for (input, value) in [
            (&self.sound.preamp, self.preferences.audio_effects.preamp_db),
            (
                &self.sound.balance,
                self.preferences.audio_effects.balance * 100.,
            ),
            (&self.sound.speed, self.preferences.playback_speed),
        ] {
            input.update(cx, |slider, cx| slider.set_value(value, window, cx));
        }
        self.send_audio(AudioCommand::Effects(
            self.preferences.audio_effects.clone(),
        ));
        self.send_audio(AudioCommand::Speed(self.preferences.playback_speed));
    }
    pub(in crate::ui) fn render_sound_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut bands = div().flex().flex_wrap().mx_neg_3();
        for (i, frequency) in FREQUENCIES.iter().enumerate() {
            bands = bands.child(
                div()
                    .w(relative(0.5))
                    .px_3()
                    .py_2()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().w(px(48.)).text_xs().child(if *frequency >= 1000. {
                        format!("{} kHz", frequency / 1000.)
                    } else {
                        format!("{frequency} Hz")
                    }))
                    .child(
                        div().flex_1().child(
                            Slider::new(&self.sound.bands[i])
                                .horizontal()
                                .disabled(!self.preferences.audio_effects.enabled),
                        ),
                    )
                    .child(
                        div()
                            .w(px(44.))
                            .text_right()
                            .text_xs()
                            .child(format!("{:+.1}", self.preferences.audio_effects.bands[i])),
                    ),
            );
        }
        let mut presets = div().flex().flex_wrap().gap_2();
        for (index, (name, gains, preamp)) in [
            ("Flat", [0.; 10], 0.),
            ("Bass", [6., 5., 3., 1., 0., 0., 0., 0., 0., 0.], -6.),
            ("Vocal", [-2., -2., -1., 0., 2., 3., 3., 1., 0., -1.], -3.),
            ("Bright", [0., 0., 0., 0., 0., 1., 2., 3., 4., 4.], -4.),
        ]
        .into_iter()
        .enumerate()
        {
            presets = presets.child(
                Button::new(("eq-preset", index))
                    .small()
                    .label(name)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.preferences.audio_effects = Effects {
                            enabled: true,
                            bands: gains,
                            preamp_db: preamp,
                            balance: this.preferences.audio_effects.balance,
                        };
                        this.initialize_sound(window, cx);
                        this.persist_modes();
                        cx.notify();
                    })),
            );
        }
        let gapless = Switch::new("gapless-playback")
            .label("Gapless playback")
            .checked(self.preferences.gapless)
            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                this.preferences.gapless = *checked;
                this.persist_modes();
                this.sync_preload();
                cx.notify();
            }));
        let album_gain = Switch::new("album-replaygain")
            .label("Use album ReplayGain when available")
            .checked(self.preferences.album_replay_gain)
            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                this.preferences.album_replay_gain = *checked;
                this.persist_modes();
                this.sync_preload();
                cx.notify();
            }));
        let equalizer = Switch::new("equalizer-enabled")
            .label("10-band equalizer")
            .checked(self.preferences.audio_effects.enabled)
            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                this.preferences.audio_effects.enabled = *checked;
                this.send_audio(AudioCommand::Effects(
                    this.preferences.audio_effects.clone(),
                ));
                this.persist_modes();
                cx.notify();
            }));
        let reset = Button::new("reset-audio-effects")
            .ghost()
            .small()
            .label("Reset sound controls")
            .on_click(cx.listener(|this, _, window, cx| {
                this.preferences.audio_effects = Effects::default();
                this.preferences.playback_speed = 1.;
                this.initialize_sound(window, cx);
                this.persist_modes();
                cx.notify();
            }));
        let balance = if self.preferences.audio_effects.balance.abs() < 0.01 {
            "Center".into()
        } else {
            format!(
                "{} {}%",
                if self.preferences.audio_effects.balance < 0. {
                    "Left"
                } else {
                    "Right"
                },
                (self.preferences.audio_effects.balance.abs() * 100.).round()
            )
        };
        div().p_5().rounded_lg().bg(cx.theme().sidebar)
            .border_1().border_color(cx.theme().border)
            .flex().flex_col().gap_4()
            .child(div().text_lg().font_weight(FontWeight::SEMIBOLD).child("Sound"))
            .child(gapless)
            .child(div().text_sm().text_color(cx.theme().muted_foreground).child(
                "Preload the next local track for continuous playback. The sleep timer takes priority over preloading."))
            .child(album_gain).child(equalizer).child(presets).child(bands)
            .child(sound_slider("Preamp", format!("{:+.1} dB", self.preferences.audio_effects.preamp_db), &self.sound.preamp, cx))
            .child(sound_slider("Balance", balance, &self.sound.balance, cx))
            .child(sound_slider("Speed", format!("{:.2}×", self.preferences.playback_speed), &self.sound.speed, cx))
            .child(div().text_sm().text_color(cx.theme().muted_foreground).child(
                "Speed changes tempo and pitch together. Preamp and balance apply even when the equalizer is off. Lower the preamp when boosting frequencies."))
            .child(reset).into_any_element()
    }
}
fn sound_slider(label: &str, value: String, input: &Entity<SliderState>, cx: &App) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(div().w(px(70.)).text_sm().child(label.to_owned()))
        .child(div().flex_1().child(Slider::new(input).horizontal()))
        .child(
            div()
                .w(px(90.))
                .text_right()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(value),
        )
        .into_any_element()
}
