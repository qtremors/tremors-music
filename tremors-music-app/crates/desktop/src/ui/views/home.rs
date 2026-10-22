// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Artwork shelves derived exclusively from the local collection.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_home(&self, cx: &mut Context<Self>) -> AnyElement {
        let columns = self.gallery_columns;
        let mut body = div()
            .id("home-content")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
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
            shortcuts = shortcuts.child(
                div()
                    .id(label)
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
            let mut shelf = div().flex().flex_col().gap_2().child(
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
            let mut strip = div().flex().items_start().gap_3();
            for (index, row) in section.rows.iter().take(columns).enumerate() {
                strip = strip.child(self.render_collection_card(
                    row.clone(),
                    format!("home-{section_index}-{index}"),
                    cx,
                ));
            }
            for _ in section.rows.len().min(columns)..columns {
                strip = strip.child(div().flex_1().min_w_0().p_2());
            }
            shelf = shelf.child(strip);
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
        .overflow_y_scrollbar()
        .into_any_element()
    }

    pub(in crate::ui) fn render_search_start(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div()
            .id("search-start")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
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
            body = body.child(
                div()
                    .id(label)
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
        body.overflow_y_scrollbar().into_any_element()
    }
}
