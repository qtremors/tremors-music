// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! A dedicated, scrollable metadata editor for local audio files.
use super::*;
use crate::ui::file_tasks::FileTask;
use tremors_core::metadata::PictureType;

impl MusicApp {
    pub(in crate::ui) fn render_metadata(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let Some(editor) = &self.metadata else {
            return div()
                .px_8()
                .child("Opening file metadata…")
                .into_any_element();
        };
        let id = editor.document.song_id;
        let saving = editor.saving;
        let simple_pictures = editor.document.tag_type == tremors_core::metadata::TagType::Mp4Ilst;
        let view = cx.entity().downgrade();
        let fields = editor.document.available_fields();
        let mut form = div().flex().flex_wrap().mx_neg_2();
        for (index, (field, input)) in editor
            .document
            .fields
            .iter()
            .zip(&editor.inputs)
            .enumerate()
        {
            let multiline = matches!(input, crate::ui::metadata::FieldInput::Paragraph(_));
            form = form.child(
                div()
                    .id(("metadata-field", index))
                    .w(relative(if multiline { 1. } else { 0.5 }))
                    .px_2()
                    .pb_4()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(field.label.clone()),
                    )
                    .child(input.element(saving, &field.label)),
            );
        }
        let mut pictures = div().flex().flex_col().gap_3();
        for (index, artwork) in editor.document.pictures.iter().enumerate() {
            let view = cx.entity().downgrade();
            let mut picture = div()
                .p_3()
                .rounded_lg()
                .bg(cx.theme().muted)
                .flex()
                .items_center()
                .gap_3()
                .child(cover_element(artwork.preview.clone(), 72., cx))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            Button::new(("picture-type", index))
                                .small()
                                .label(format!("{:?}", artwork.picture.pic_type()))
                                .disabled(saving || simple_pictures)
                                .dropdown_menu(move |mut menu, _, _| {
                                    for kind in [
                                        PictureType::CoverFront,
                                        PictureType::CoverBack,
                                        PictureType::Leaflet,
                                        PictureType::Media,
                                        PictureType::Artist,
                                        PictureType::Band,
                                        PictureType::Composer,
                                        PictureType::Illustration,
                                        PictureType::BandLogo,
                                        PictureType::PublisherLogo,
                                        PictureType::Other,
                                        PictureType::Icon,
                                        PictureType::OtherIcon,
                                        PictureType::LeadArtist,
                                        PictureType::Conductor,
                                        PictureType::Lyricist,
                                        PictureType::RecordingLocation,
                                        PictureType::DuringRecording,
                                        PictureType::DuringPerformance,
                                        PictureType::ScreenCapture,
                                        PictureType::BrightFish,
                                    ] {
                                        let view = view.clone();
                                        menu = menu.item(
                                            PopupMenuItem::new(format!("{kind:?}")).on_click(
                                                move |_, _, cx| {
                                                    let _ = view.update(cx, |this, cx| {
                                                        if let Some(editor) = &mut this.metadata
                                                            && !editor.saving
                                                            && let Some(art) = editor
                                                                .document
                                                                .pictures
                                                                .get_mut(index)
                                                        {
                                                            art.picture.set_pic_type(kind);
                                                            editor.dirty = true;
                                                            cx.notify();
                                                        }
                                                    });
                                                },
                                            ),
                                        );
                                    }
                                    menu
                                }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Description"),
                        )
                        .child(
                            inputs::field(&editor.picture_descriptions[index])
                                .aria_label("Artwork description")
                                .disabled(saving || simple_pictures),
                        ),
                );
            for (label, delta, icon) in [
                ("picture-up", -1_isize, IconName::ChevronUp),
                ("picture-down", 1, IconName::ChevronDown),
            ] {
                picture = picture.child(
                    Button::new((label, index))
                        .ghost()
                        .small()
                        .icon(Icon::new(icon))
                        .disabled(
                            saving
                                || (delta < 0 && index == 0)
                                || (delta > 0 && index + 1 == editor.document.pictures.len()),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(editor) = &mut this.metadata
                                && !editor.saving
                            {
                                let target = index.saturating_add_signed(delta);
                                if target < editor.document.pictures.len() {
                                    editor.document.pictures.swap(index, target);
                                    editor.picture_descriptions.swap(index, target);
                                    editor.dirty = true;
                                    cx.notify();
                                }
                            }
                        })),
                );
            }
            picture = picture.child(
                Button::new(("remove-picture", index))
                    .ghost()
                    .small()
                    .label("Remove")
                    .disabled(saving)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(editor) = &mut this.metadata
                            && !editor.saving
                            && index < editor.document.pictures.len()
                        {
                            editor.document.pictures.remove(index);
                            editor.picture_descriptions.remove(index);
                            editor.dirty = true;
                            cx.notify();
                        }
                    })),
            );
            pictures = pictures.child(picture);
        }
        let toolbar = div()
            .px_8()
            .pb_4()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .child(
                Button::new("save-metadata")
                    .primary()
                    .label(if saving { "Saving…" } else { "Save to file" })
                    .disabled(saving || !editor.dirty)
                    .on_click(cx.listener(|this, _, _, cx| this.save_metadata(cx))),
            )
            .child(
                Button::new("reload-metadata")
                    .label("Revert / reload")
                    .disabled(saving)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.send_library(LibraryCommand::ReadMetadata(id));
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(if editor.message.is_empty() {
                        if editor.dirty {
                            "Unsaved changes".to_owned()
                        } else {
                            "Changes are written only when you save.".to_owned()
                        }
                    } else {
                        editor.message.clone()
                    }),
            );
        let body = div().id("metadata-scroll").flex_1().min_h_0().overflow_y_scroll().px_8().pb_8().flex().flex_col().gap_5()
            .child(div().text_xs().text_color(cx.theme().muted_foreground).child(editor.document.path.display().to_string()))
            .when(editor.document.has_sidecar_lyrics, |d| d.child(div().text_sm().child("This track has a local lyric sidecar. It takes priority over embedded lyrics during playback.")))
            .child(form)
            .child(Button::new("add-metadata-field").label("Add extended field").disabled(saving).dropdown_menu(move |mut menu,_,_| {
                for (index, (_,label)) in fields.iter().enumerate() {
                    let view = view.clone(); menu = menu.item(PopupMenuItem::new(label.clone()).on_click(move |_,window,cx| { let _ = view.update(cx, |this,cx| this.add_metadata_field(index,window,cx)); }));
                } menu
            }))
            .child(div().text_lg().font_weight(FontWeight::SEMIBOLD).child("Embedded artwork"))
            .when(simple_pictures, |d| d.child("This format stores cover images without picture roles or descriptions."))
            .child(pictures)
            .child(Button::new("add-metadata-picture").label("Add artwork").disabled(saving || self.picking_file)
                .on_click(cx.listener(move |this,_,_,cx| { this.pick_file(FileTask::MetadataPicture(id)); cx.notify(); })))
            .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Available fields depend on the file format. Audio properties are read-only. Playback pauses if this track is open or preloaded while saving."))
            .track_scroll(&self.page_scroll).vertical_scrollbar(&self.page_scroll);
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(body)
            .into_any_element()
    }
}
