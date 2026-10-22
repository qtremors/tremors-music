// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Artwork shelves derived exclusively from the local collection.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_home(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let columns = self.gallery_columns;
        let card_width =
            ((f32::from(window.viewport_size().width) - 224. - 48. - (columns - 1) as f32 * 12.)
                / columns as f32)
                .max(140.);
        let mut body = div()
            .id("home-content")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .restrict_scroll_to_axis()
            .px_6()
            .pb_6()
            .flex()
            .flex_col()
            .gap_6();
        let mut shortcuts = div().px_2().flex().gap_3();
        for (label, subtitle, page, icon) in [
            (
                "Favorites",
                "Keep your favorites close",
                Page::Favorites,
                IconName::Heart,
            ),
            (
                "Most played",
                "Back to your regulars",
                Page::MostPlayed,
                IconName::TrendingUp,
            ),
            (
                "Playlists",
                "A mix for every mood",
                Page::Playlists,
                IconName::ListMusic,
            ),
        ] {
            let focus = self.collection_focus(label, scrolling::FocusTarget::Body(0), window, cx);
            let keyboard_page = page.clone();
            shortcuts = shortcuts.child(
                div()
                    .id(label)
                    .track_focus(&focus)
                    .tab_index(0)
                    .key_context("CollectionCard")
                    .role(gpui_kit::accesskit::Role::Button)
                    .aria_label(label)
                    .focus(|d| d.bg(cx.theme().muted))
                    .on_action(
                        cx.listener(move |this, _: &ActivateCollection, window, cx| {
                            this.navigate(keyboard_page.clone(), window, cx)
                        }),
                    )
                    .flex_1()
                    .min_w_0()
                    .p_4()
                    .rounded_lg()
                    .bg(cx.theme().sidebar)
                    .border_1()
                    .border_color(cx.theme().border)
                    .flex()
                    .items_center()
                    .gap_3()
                    .cursor_pointer()
                    .hover(|d| d.bg(cx.theme().muted))
                    .child(Icon::new(icon).size_6().text_color(cx.theme().primary))
                    .child(
                        div()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(label),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(subtitle),
                            ),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.navigate(page.clone(), window, cx)
                    })),
            );
        }
        body = body.child(shortcuts);
        for (section_index, section) in self.home_sections.iter().enumerate() {
            let page = section.page.clone();
            let scroll = self.home_scroll.entry(section.title).or_default().clone();
            let mut shelf = div().flex_shrink_0().flex().flex_col().gap_2().child(
                div()
                    .px_2()
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .child(section.title),
                    )
                    .children(
                        [
                            (-1., IconName::ChevronLeft, "Previous collections"),
                            (1., IconName::ChevronRight, "More collections"),
                        ]
                        .into_iter()
                        .map(|(direction, icon, label)| {
                            let handle = scroll.clone();
                            Button::new((
                                if direction < 0. {
                                    "shelf-prev"
                                } else {
                                    "shelf-next"
                                },
                                section_index,
                            ))
                            .ghost()
                            .small()
                            .icon(Icon::new(icon))
                            .tooltip(label)
                            .accessibility_label(label)
                            .disabled(
                                section.rows.len() <= columns
                                    || (direction < 0. && scroll.offset().x >= px(-0.5))
                                    || (direction > 0.
                                        && scroll.max_offset().x > px(0.)
                                        && -scroll.offset().x >= scroll.max_offset().x - px(0.5)),
                            )
                            .on_click(cx.listener(
                                move |_, _, _, cx| {
                                    let mut offset = handle.offset();
                                    offset.x = (offset.x - px(direction * (card_width + 12.) * 2.))
                                        .clamp(-handle.max_offset().x, px(0.));
                                    handle.set_offset(offset);
                                    cx.notify();
                                },
                            ))
                        }),
                    )
                    .child(
                        Button::new(("shelf-all", section_index))
                            .ghost()
                            .small()
                            .label("See all")
                            .icon(Icon::new(IconName::ChevronRight))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.navigate(page.clone(), window, cx)
                            })),
                    ),
            );
            let mut strip = div()
                .id(("home-strip", section_index))
                .w_full()
                .flex()
                .items_start()
                .gap_3()
                .pb_3()
                .overflow_x_scroll()
                .restrict_scroll_to_axis()
                .track_scroll(&scroll);
            for (index, row) in section.rows.iter().enumerate() {
                strip = strip.child(div().w(px(card_width)).flex_shrink_0().child(
                    self.render_collection_card(
                        row.clone(),
                        format!("home-{section_index}-{index}"),
                        scrolling::FocusTarget::Shelf {
                            scroll: scroll.clone(),
                            item: index,
                            section: section_index,
                        },
                        window,
                        cx,
                    ),
                ));
            }
            shelf = shelf.child(strip.horizontal_scrollbar(&scroll));
            body = body.child(shelf);
        }
        body.child(
            div()
                .px_2()
                .pt_4()
                .border_t_1()
                .border_color(cx.theme().border)
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Collected by you. Stored on your device. Ready whenever you are."),
        )
        .track_scroll(&self.page_scroll)
        .vertical_scrollbar(&self.page_scroll)
        .into_any_element()
    }

    pub(in crate::ui) fn render_search_start(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut body = div()
            .id("search-start")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .restrict_scroll_to_axis()
            .px_8()
            .pb_8()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Explore your library"),
            );
        for (label, description, page, icon) in [
            (
                "Albums",
                "From the first track to the last",
                Page::Albums,
                IconName::Disc3,
            ),
            (
                "Artists",
                "The people behind your music",
                Page::Artists,
                IconName::Users,
            ),
            (
                "Genres",
                "Find the sound you are in the mood for",
                Page::Genres,
                IconName::Music,
            ),
            (
                "Playlists",
                "Your collection, your way",
                Page::Playlists,
                IconName::ListMusic,
            ),
        ] {
            let focus = self.collection_focus(
                label,
                scrolling::FocusTarget::Body(match label {
                    "Albums" => 1,
                    "Artists" => 2,
                    "Genres" => 3,
                    _ => 4,
                }),
                window,
                cx,
            );
            let keyboard_page = page.clone();
            body = body.child(
                div()
                    .id(label)
                    .track_focus(&focus)
                    .tab_index(0)
                    .key_context("CollectionCard")
                    .role(gpui_kit::accesskit::Role::Button)
                    .aria_label(label)
                    .focus(|d| d.bg(cx.theme().muted))
                    .on_action(
                        cx.listener(move |this, _: &ActivateCollection, window, cx| {
                            this.navigate(keyboard_page.clone(), window, cx)
                        }),
                    )
                    .p_5()
                    .rounded_lg()
                    .bg(cx.theme().sidebar)
                    .flex()
                    .items_center()
                    .gap_4()
                    .cursor_pointer()
                    .hover(|d| d.bg(cx.theme().muted))
                    .child(Icon::new(icon).size_6().text_color(cx.theme().primary))
                    .child(
                        div()
                            .flex_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(label),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(description),
                            ),
                    )
                    .child(Icon::new(IconName::ChevronRight))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.navigate(page.clone(), window, cx)
                    })),
            );
        }
        body.track_scroll(&self.page_scroll)
            .vertical_scrollbar(&self.page_scroll)
            .into_any_element()
    }
}
