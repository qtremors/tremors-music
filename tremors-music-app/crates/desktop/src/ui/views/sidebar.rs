// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Library navigation and local playlist shortcuts.

use super::*;

fn brand_mark() -> Arc<Image> {
    static MARK: std::sync::OnceLock<Arc<Image>> = std::sync::OnceLock::new();
    MARK.get_or_init(|| {
        Arc::new(Image::from_bytes(
            ImageFormat::Png,
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../assets/TremorsMusic.png"
            ))
            .to_vec(),
        ))
    })
    .clone()
}

impl MusicApp {
    fn sidebar_link(
        &self,
        label: &str,
        page: Page,
        icon: IconName,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.page == page
            || matches!(
                (&self.page, &page),
                (Page::Album(..), Page::Albums)
                    | (Page::Artist(_), Page::Artists)
                    | (Page::Genre(_), Page::Genres)
            );
        Button::new(SharedString::from(match &page {
            Page::Playlist(id) => format!("nav-playlist-{id}"),
            _ => format!("nav-{label}"),
        }))
        .ghost()
        .w_full()
        .accessibility_label(label.to_owned())
        .child(
            div()
                .w_full()
                .min_w_0()
                .flex()
                .items_center()
                .gap_2()
                .child(Icon::new(icon).size_4().text_color(if selected {
                    cx.theme().primary
                } else {
                    cx.theme().muted_foreground
                }))
                .child(div().flex_1().truncate().child(label.to_owned())),
        )
        .when(selected, |b| b.bg(cx.theme().muted))
        .selected(selected)
        .on_click(cx.listener(move |this, _, window, cx| this.navigate(page.clone(), window, cx)))
        .into_any_element()
    }

    pub(in crate::ui) fn render_sidebar(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut navigation = div()
            .id("library-navigation")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .px_3()
            .flex()
            .flex_col()
            .gap_1()
            .child(self.sidebar_link("Home", Page::Home, IconName::Music, cx))
            .child(self.sidebar_link("Search", Page::Search, IconName::Search, cx))
            .child(
                div()
                    .px_3()
                    .mt_6()
                    .mb_2()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(cx.theme().muted_foreground)
                    .child("Library"),
            );
        for (label, page, icon) in [
            ("Recently added", Page::Recent, IconName::Clock),
            ("Recently played", Page::RecentlyPlayed, IconName::Clock),
            ("Artists", Page::Artists, IconName::Users),
            ("Albums", Page::Albums, IconName::Disc3),
            ("Songs", Page::Library, IconName::Music),
            ("Genres", Page::Genres, IconName::Disc3),
            ("Favorites", Page::Favorites, IconName::Heart),
            ("Most played", Page::MostPlayed, IconName::TrendingUp),
        ] {
            navigation = navigation.child(self.sidebar_link(label, page, icon, cx));
        }
        navigation = navigation
            .child(
                div()
                    .mt_6()
                    .mb_1()
                    .pl_3()
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(cx.theme().muted_foreground)
                            .child("Playlists"),
                    )
                    .child(
                        Button::new("sidebar-new-playlist")
                            .ghost()
                            .small()
                            .icon(Icon::new(IconName::Plus))
                            .tooltip("New playlist")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.playlist_name.update(cx, |input, cx| {
                                    input.set_value("", window, cx);
                                    input.focus(window, cx);
                                });
                                this.overlay = Some(Overlay::CreatePlaylist);
                                cx.notify();
                            })),
                    ),
            )
            .child(self.sidebar_link("All playlists", Page::Playlists, IconName::ListMusic, cx));
        for playlist in &self.snapshot.playlists {
            navigation = navigation.child(self.sidebar_link(
                &playlist.name,
                Page::Playlist(playlist.id),
                IconName::ListMusic,
                cx,
            ));
        }
        if self.snapshot.playlists.is_empty() {
            navigation = navigation.child(
                div()
                    .px_3()
                    .py_2()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Your mixes start here."),
            );
        }
        div()
            .w(px(224.))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .bg(cx.theme().sidebar)
            .border_r_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .px_6()
                    .py_6()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        img(brand_mark())
                            .size(px(24.))
                            .flex_shrink_0()
                            .object_fit(ObjectFit::Contain),
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .child("Tremors Music"),
                    ),
            )
            .child(navigation.overflow_y_scrollbar())
            .child(
                div()
                    .p_3()
                    .mt_3()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(self.sidebar_link("Music folders", Page::Folders, IconName::Folder, cx))
                    .child(self.sidebar_link("Settings", Page::Settings, IconName::Settings, cx))
                    .child(
                        div()
                            .px_3()
                            .pt_3()
                            .pb_1()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{} songs · On this device",
                                self.snapshot.songs.len()
                            )),
                    ),
            )
            .into_any_element()
    }
}
