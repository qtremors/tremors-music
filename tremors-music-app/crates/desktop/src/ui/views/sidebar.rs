// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Primary navigation and playlist shortcuts.

use super::*;

impl MusicApp {
    pub(in crate::ui) fn render_sidebar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut sidebar = div()
            .id("sidebar")
            .overflow_y_scroll()
            .w(px(220.))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .gap_1()
            .p_4()
            .bg(cx.theme().sidebar)
            .border_r_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .mb_1()
                    .child("Tremors Music"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .mb_4()
                    .child("YOUR MUSIC. YOUR SPACE."),
            );
        for (label, page, icon) in [
            ("Search", Page::Search, IconName::Search),
            ("Songs", Page::Library, IconName::Music),
            ("Albums", Page::Albums, IconName::Disc3),
            ("Artists", Page::Artists, IconName::Users),
            ("Genres", Page::Genres, IconName::Disc3),
            ("Playlists", Page::Playlists, IconName::ListMusic),
            ("Favorites", Page::Favorites, IconName::Heart),
            ("Recently added", Page::Recent, IconName::Clock),
            ("Most played", Page::MostPlayed, IconName::TrendingUp),
            ("Queue", Page::Queue, IconName::ListMusic),
            ("Music folders", Page::Folders, IconName::Folder),
        ] {
            sidebar = sidebar.child(
                div()
                    .id(label)
                    .w_full()
                    .h(px(30.))
                    .px_3()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .when(self.page == page, |d| d.bg(cx.theme().muted))
                    .hover(|d| d.bg(cx.theme().muted))
                    .child(Icon::new(icon).size_4())
                    .child(label)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.navigate(page.clone(), window, cx)
                    })),
            );
        }
        sidebar = sidebar.child(
            div()
                .mt_4()
                .mb_2()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(cx.theme().muted_foreground)
                .child("PLAYLISTS"),
        );
        let mut playlists = div()
            .id("sidebar-playlists")
            .min_h(px(80.))
            .flex()
            .flex_col()
            .gap_1();
        for pl in &self.snapshot.playlists {
            let id = pl.id;
            playlists = playlists.child(
                div()
                    .id(("sidebar-playlist", id as usize))
                    .w_full()
                    .h(px(30.))
                    .px_3()
                    .rounded_md()
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .when(self.page == Page::Playlist(id), |d| d.bg(cx.theme().muted))
                    .hover(|d| d.bg(cx.theme().muted))
                    .child(Icon::new(IconName::ListMusic).size_4())
                    .child(pl.name.clone())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.navigate(Page::Playlist(id), window, cx)
                    })),
            );
        }
        sidebar = sidebar.child(playlists);
        if !matches!(self.page, Page::Playlist(_)) {
            sidebar = sidebar
                .child(Input::new(&self.playlist_name).small())
                .child(
                    Button::new("new-playlist")
                        .ghost()
                        .label("Create playlist")
                        .icon(Icon::new(IconName::Plus))
                        .w_full()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.save_playlist(window, cx)),
                        ),
                );
        }
        let _ = window;
        sidebar
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .mt_3()
                    .child(format!(
                        "v{} · {} songs",
                        env!("CARGO_PKG_VERSION"),
                        self.snapshot.songs.len()
                    )),
            )
            .into_any_element()
    }
}
