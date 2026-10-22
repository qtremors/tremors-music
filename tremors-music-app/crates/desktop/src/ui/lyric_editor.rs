// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Local lyric editing and per-track display offsets.
use super::*;
use gpui_kit::component::input::TextareaState;
pub(super) struct Editor {
    pub document: tremors_core::lyrics::file::Document,
    input: Entity<TextareaState>,
    pub dirty: bool,
    pub saving: bool,
    pub message: String,
    timed_lines: usize,
    _subscription: Subscription,
}
impl MusicApp {
    pub(in crate::ui) fn draft_dirty(&self) -> bool {
        self.metadata.as_ref().is_some_and(|e| e.dirty)
            || self.lyric_editor.as_ref().is_some_and(|e| e.dirty)
    }
    pub(in crate::ui) fn receive_lyrics(
        &mut self,
        document: tremors_core::lyrics::file::Document,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.page != Page::LyricsEditor(document.song_id) {
            return;
        }
        let saved = self.lyric_editor.as_ref().is_some_and(|e| e.saving);
        if saved {
            self.status = "Lyrics saved beside your music.".into();
        }
        let input = cx.new(|cx| {
            let mut input = TextareaState::new(window, cx);
            input.set_value(document.text.clone(), window, cx);
            input
        });
        let subscription = cx.subscribe(&input, |this, input, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change)
                && let Some(editor) = &mut this.lyric_editor
            {
                editor.dirty = input.read(cx).value().as_ref() != editor.document.text;
                editor.timed_lines = parse_lrc(input.read(cx).value().as_ref()).len();
                editor.message.clear();
                cx.notify();
            }
        });
        let timed_lines = parse_lrc(&document.text).len();
        self.lyric_editor = Some(Editor {
            document,
            timed_lines,
            input,
            dirty: false,
            saving: false,
            message: if saved {
                "Lyrics saved beside your music.".into()
            } else {
                String::new()
            },
            _subscription: subscription,
        });
    }
    pub(in crate::ui) fn lyric_offset(&self) -> f64 {
        self.queue
            .current()
            .and_then(|s| self.preferences.lyric_offsets.get(&s.path))
            .copied()
            .unwrap_or(0) as f64
            / 1000.
    }
    pub(in crate::ui) fn lyric_position(&self) -> f64 {
        self.position - self.lyric_offset()
    }
    fn shift_lyrics(&mut self, delta: i32, cx: &mut Context<Self>) {
        if let Some(song) = self.queue.current() {
            let offset = self
                .preferences
                .lyric_offsets
                .entry(song.path.clone())
                .or_default();
            *offset = (*offset + delta).clamp(-10_000, 10_000);
            self.persist_modes();
            self.lyric_index = None;
            cx.notify();
        }
    }
    pub(in crate::ui) fn render_lyric_tools(&self, cx: &mut Context<Self>) -> AnyElement {
        let song_id = self.queue.current().map(|s| s.id);
        div()
            .flex_shrink_0()
            .px_8()
            .pb_2()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(
                Button::new("edit-local-lyrics")
                    .ghost()
                    .small()
                    .label("Edit lyrics")
                    .disabled(song_id.is_none())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if let Some(id) = song_id {
                            this.navigate(Page::LyricsEditor(id), window, cx);
                        }
                    })),
            )
            .when(!self.lyric_lines.is_empty(), |d| {
                d.child(
                    Button::new("lyrics-earlier")
                        .ghost()
                        .small()
                        .label("−")
                        .tooltip("Show lyrics 100 ms earlier")
                        .on_click(cx.listener(|this, _, _, cx| this.shift_lyrics(-100, cx))),
                )
                .child(
                    Button::new("lyrics-offset-reset")
                        .ghost()
                        .small()
                        .label(format!("{:+.1} s", self.lyric_offset()))
                        .tooltip("Lyric delay · click to reset")
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(song) = this.queue.current() {
                                this.preferences.lyric_offsets.remove(&song.path);
                                this.persist_modes();
                                this.lyric_index = None;
                                cx.notify();
                            }
                        })),
                )
                .child(
                    Button::new("lyrics-later")
                        .ghost()
                        .small()
                        .label("+")
                        .tooltip("Show lyrics 100 ms later")
                        .on_click(cx.listener(|this, _, _, cx| this.shift_lyrics(100, cx))),
                )
                .child(div().flex_1())
                .child(
                    Button::new("follow-lyrics")
                        .ghost()
                        .small()
                        .label(if self.lyric_follow {
                            "Following lyrics"
                        } else {
                            "Resume following"
                        })
                        .selected(self.lyric_follow)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.lyric_follow = !this.lyric_follow;
                            if this.lyric_follow
                                && let Some(index) = this.lyric_index
                            {
                                this.lyrics_scroll.scroll_to_reveal_item(index);
                            }
                            cx.notify();
                        })),
                )
            })
            .into_any_element()
    }
    pub(in crate::ui) fn render_lyric_editor(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(editor) = &self.lyric_editor else {
            return div()
                .px_8()
                .child("Opening local lyrics…")
                .into_any_element();
        };
        let count = editor.timed_lines;
        let height = (f32::from(window.viewport_size().height) - 440.).clamp(180., 320.);
        let save = Button::new("save-local-lyrics")
            .primary()
            .label(if editor.saving {
                "Saving…"
            } else {
                "Save lyrics"
            })
            .disabled(editor.saving || !editor.dirty)
            .on_click(cx.listener(|this, _, _, cx| {
                if let Some(editor) = &mut this.lyric_editor {
                    let mut document = editor.document.clone();
                    document.text = editor.input.read(cx).value().to_string();
                    editor.saving = true;
                    this.send_library(LibraryCommand::SaveLyrics(document));
                    cx.notify();
                }
            }));
        let reopen = Button::new("revert-local-lyrics")
            .label("Reopen file")
            .disabled(editor.saving)
            .on_click(cx.listener(|this, _, _, cx| {
                if let Some(editor) = &this.lyric_editor {
                    this.send_library(LibraryCommand::ReadLyrics(editor.document.song_id));
                    cx.notify();
                }
            }));
        let actions = div()
            .flex()
            .flex_wrap()
            .gap_2()
            .items_center()
            .child(save)
            .child(reopen)
            .child(div().text_xs().child(if count > 0 {
                format!("{count} timed lines")
            } else {
                "Plain lyrics".into()
            }));
        div().id("lyric-editor").flex_1().min_h_0().px_8().pb_6()
            .overflow_y_scroll().flex().flex_col().gap_3()
            .child(div().text_sm().text_color(cx.theme().muted_foreground)
                .child(editor.document.path.display().to_string()))
            .child("Edit plain text, LRC timestamps, enhanced LRC word timing, or TTML. Saving changes this lyric sidecar only.")
            .child(actions)
            .when(!editor.message.is_empty(), |d| d.child(editor.message.clone()))
            .child(inputs::paragraph(&editor.input, height).aria_label("Local lyric text").disabled(editor.saving))
            .track_scroll(&self.page_scroll).vertical_scrollbar(&self.page_scroll)
            .into_any_element()
    }
}
