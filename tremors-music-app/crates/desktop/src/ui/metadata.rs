// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Metadata form state. Reading and writing music files belongs to the library worker.
use super::*;
use gpui_kit::component::input::{Textarea, TextareaState};
use tremors_core::metadata::MetadataDocument;

pub(super) enum FieldInput {
    Line(Entity<InputState>),
    Paragraph(Entity<TextareaState>),
}
impl FieldInput {
    pub(super) fn value(&self, cx: &App) -> String {
        match self {
            Self::Line(input) => input.read(cx).value().to_string(),
            Self::Paragraph(input) => input.read(cx).value().to_string(),
        }
    }
    pub(super) fn element(&self, disabled: bool) -> AnyElement {
        match self {
            Self::Line(input) => Input::new(input).disabled(disabled).into_any_element(),
            Self::Paragraph(input) => Textarea::new(input)
                .disabled(disabled)
                .h(px(160.))
                .into_any_element(),
        }
    }
}
pub(super) struct MetadataEditor {
    pub document: MetadataDocument,
    pub inputs: Vec<FieldInput>,
    pub picture_descriptions: Vec<Entity<InputState>>,
    pub dirty: bool,
    pub saving: bool,
    pub message: String,
    pub subscriptions: Vec<Subscription>,
}
impl MusicApp {
    pub(in crate::ui) fn metadata_busy(&self) -> bool {
        self.metadata.as_ref().is_some_and(|e| e.saving)
    }
    pub(in crate::ui) fn open_metadata(
        &mut self,
        id: i64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate(Page::Metadata(id), window, cx);
    }
    pub(in crate::ui) fn metadata_input(
        &mut self,
        value: &str,
        multiline: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (FieldInput, Subscription) {
        if multiline {
            let input = cx.new(|cx| {
                let mut input = TextareaState::new(window, cx);
                input.set_value(value.to_owned(), window, cx);
                input
            });
            let subscription = cx.subscribe_in(&input, window, |this, _, event, _, cx| {
                if matches!(event, InputEvent::Change) {
                    if let Some(editor) = &mut this.metadata {
                        editor.dirty = true;
                        editor.message.clear();
                    }
                    cx.notify();
                }
            });
            (FieldInput::Paragraph(input), subscription)
        } else {
            let input = cx.new(|cx| {
                let mut input = InputState::new(window, cx);
                input.set_value(value.to_owned(), window, cx);
                input
            });
            let subscription = cx.subscribe_in(&input, window, |this, _, event, _, cx| {
                if matches!(event, InputEvent::Change) {
                    if let Some(editor) = &mut this.metadata {
                        editor.dirty = true;
                        editor.message.clear();
                    }
                    cx.notify();
                }
            });
            (FieldInput::Line(input), subscription)
        }
    }
    pub(in crate::ui) fn receive_metadata(
        &mut self,
        document: MetadataDocument,
        saved: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.page != Page::Metadata(document.song_id) {
            return;
        }
        let mut inputs = Vec::new();
        let mut subscriptions = Vec::new();
        for field in &document.fields {
            let multiline = field.label.contains("lyrics")
                || field.label == "Comment"
                || field.value.contains('\n');
            let (input, subscription) = self.metadata_input(&field.value, multiline, window, cx);
            inputs.push(input);
            subscriptions.push(subscription);
        }
        let mut picture_descriptions = Vec::new();
        for picture in &document.pictures {
            let (input, subscription) = self.metadata_input(
                picture.picture.description().unwrap_or_default(),
                false,
                window,
                cx,
            );
            let FieldInput::Line(input) = input else {
                unreachable!()
            };
            picture_descriptions.push(input);
            subscriptions.push(subscription);
        }
        self.metadata = Some(MetadataEditor {
            document,
            inputs,
            subscriptions,
            picture_descriptions,
            dirty: false,
            saving: false,
            message: if saved {
                "Saved to your music file.".into()
            } else {
                String::new()
            },
        });
    }
    pub(in crate::ui) fn add_metadata_field(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(editor) = &mut self.metadata else {
            return;
        };
        if editor.saving {
            return;
        }
        let Some((key, _)) = editor.document.available_fields().get(index).cloned() else {
            return;
        };
        if let Err(error) = editor.document.add_field(key) {
            editor.message = error.to_string();
            return;
        }
        let (input, subscription) = self.metadata_input("", false, window, cx);
        let editor = self.metadata.as_mut().unwrap();
        editor.inputs.push(input);
        editor.subscriptions.push(subscription);
        editor.dirty = true;
        cx.notify();
    }
    pub(in crate::ui) fn append_picture(
        &mut self,
        id: i64,
        mut picture: tremors_core::metadata::Artwork,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.page != Page::Metadata(id) || self.metadata_busy() {
            return;
        }
        if self
            .metadata
            .as_ref()
            .is_some_and(|e| e.document.tag_type == tremors_core::metadata::TagType::Mp4Ilst)
        {
            picture
                .picture
                .set_pic_type(tremors_core::metadata::PictureType::Other);
        }
        let (input, subscription) = self.metadata_input(
            picture.picture.description().unwrap_or_default(),
            false,
            window,
            cx,
        );
        if let Some(editor) = &mut self.metadata {
            let FieldInput::Line(input) = input else {
                return;
            };
            editor.picture_descriptions.push(input);
            editor.subscriptions.push(subscription);
            editor.document.pictures.push(picture);
            editor.dirty = true;
        }
    }
    pub(in crate::ui) fn save_metadata(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &mut self.metadata else {
            return;
        };
        if editor.saving || !editor.dirty {
            return;
        }
        let mut document = editor.document.clone();
        for (field, input) in document.fields.iter_mut().zip(&editor.inputs) {
            field.value = input.value(cx);
        }
        for (picture, description) in document
            .pictures
            .iter_mut()
            .zip(&editor.picture_descriptions)
        {
            let value = description.read(cx).value().to_string();
            picture
                .picture
                .set_description(if value.is_empty() { None } else { Some(value) });
        }
        editor.saving = true;
        editor.message = "Saving metadata…".into();
        let release = if self
            .queue
            .current()
            .is_some_and(|s| s.id == document.song_id)
        {
            let (tx, rx) = mpsc::channel();
            self.generation += 1;
            self.send_audio(AudioCommand::Release(tx));
            self.loaded = false;
            self.loading = false;
            self.is_playing = false;
            Some(rx)
        } else {
            None
        };
        self.send_library(LibraryCommand::SaveMetadata(Box::new(document), release));
        cx.notify();
    }
}
