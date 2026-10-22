// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Preserve browsing positions and share keyboard scrolling across page types.
use super::*;
#[derive(Clone)]
pub(super) struct PageScroll {
    body: ScrollHandle,
    table: UniformListScrollHandle,
    gallery: UniformListScrollHandle,
    query: String,
    artist_albums: bool,
}
impl Default for PageScroll {
    fn default() -> Self {
        Self {
            body: ScrollHandle::new(),
            table: UniformListScrollHandle::new(),
            gallery: UniformListScrollHandle::new(),
            query: String::new(),
            artist_albums: true,
        }
    }
}
pub(super) enum FocusTarget {
    Gallery(usize),
    Table(usize),
    Body(usize),
    Shelf {
        scroll: ScrollHandle,
        item: usize,
        section: usize,
    },
}
struct CollectionFocus {
    handle: FocusHandle,
    _subscription: Subscription,
}
impl MusicApp {
    pub(in crate::ui) fn collection_focus(
        &self,
        key: &str,
        target: FocusTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> FocusHandle {
        let view = cx.entity().downgrade();
        window
            .use_keyed_state(
                SharedString::from(format!("focus-{key}")),
                cx,
                move |window, cx| {
                    let handle = cx.focus_handle().tab_stop(true).tab_index(0);
                    let subscription = cx.on_focus(&handle, window, move |_, _, cx| {
                        let _ = view.update(cx, |this, cx| {
                            match &target {
                                FocusTarget::Gallery(index) => {
                                    this.gallery_scroll.scroll_to_item_strict(
                                        *index / this.gallery_columns,
                                        ScrollStrategy::Center,
                                    );
                                }
                                FocusTarget::Table(index) => this
                                    .table_scroll
                                    .scroll_to_item_strict(*index, ScrollStrategy::Center),
                                FocusTarget::Body(index) => this.page_scroll.scroll_to_item(*index),
                                FocusTarget::Shelf {
                                    scroll,
                                    item,
                                    section,
                                } => {
                                    scroll.scroll_to_item(*item);
                                    this.page_scroll.scroll_to_item(section + 1);
                                }
                            }
                            cx.notify();
                        });
                    });
                    CollectionFocus {
                        handle,
                        _subscription: subscription,
                    }
                },
            )
            .read(cx)
            .handle
            .clone()
    }
    pub(in crate::ui) fn remember_scroll(&mut self) {
        self.saved_scroll.retain(|(page, _)| *page != self.page);
        self.saved_scroll.push((
            self.page.clone(),
            PageScroll {
                body: self.page_scroll.clone(),
                table: self.table_scroll.clone(),
                gallery: self.gallery_scroll.clone(),
                query: self.query.clone(),
                artist_albums: self.show_artist_albums,
            },
        ));
        if self.saved_scroll.len() > 64 {
            self.saved_scroll.remove(0);
        }
    }
    pub(in crate::ui) fn restore_scroll(&mut self) {
        let scroll = self
            .saved_scroll
            .iter()
            .find(|(page, _)| *page == self.page)
            .map(|(_, scroll)| scroll.clone())
            .unwrap_or_default();
        self.query = if self.page == Page::Search {
            scroll.query
        } else {
            String::new()
        };
        self.show_artist_albums = scroll.artist_albums;
        self.page_scroll = scroll.body;
        self.table_scroll = scroll.table;
        self.gallery_scroll = scroll.gallery;
    }
    pub(in crate::ui) fn keyboard_scroll(
        &mut self,
        direction: f32,
        edge: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus, cx);
        if self.page == Page::Lyrics || self.immersive {
            self.lyric_follow = false;
            if edge {
                if direction > 0. {
                    self.lyrics_scroll.scroll_to_end();
                } else {
                    self.lyrics_scroll.scroll_to(ListOffset::default());
                }
            } else {
                self.lyrics_scroll.scroll_by(px(direction * 320.));
            }
        } else {
            let gallery = matches!(
                self.page,
                Page::Albums | Page::Artists | Page::Genres | Page::Playlists
            ) || (matches!(self.page, Page::Artist(_)) && self.show_artist_albums);
            if edge && (gallery || !self.rows.is_empty()) {
                let (list, count) = if gallery {
                    let count = if matches!(self.page, Page::Artist(_)) && self.show_artist_albums {
                        self.artist_collections.len()
                    } else {
                        self.rows.len()
                    };
                    (&self.gallery_scroll, count.div_ceil(self.gallery_columns))
                } else {
                    (&self.table_scroll, self.rows.len())
                };
                if count > 0 {
                    list.scroll_to_item_strict(
                        if direction > 0. { count - 1 } else { 0 },
                        if direction > 0. {
                            ScrollStrategy::Bottom
                        } else {
                            ScrollStrategy::Top
                        },
                    );
                }
                cx.notify();
                return;
            }
            let handle = if gallery {
                self.gallery_scroll.0.borrow().base_handle.clone()
            } else if !self.rows.is_empty() {
                self.table_scroll.0.borrow().base_handle.clone()
            } else {
                self.page_scroll.clone()
            };
            let mut offset = handle.offset();
            let height = handle.bounds().size.height;
            offset.y = if edge {
                if direction > 0. {
                    -handle.max_offset().y
                } else {
                    px(0.)
                }
            } else {
                (offset.y - height * direction * 0.85).clamp(-handle.max_offset().y, px(0.))
            };
            handle.set_offset(offset);
        }
        cx.notify();
    }
}
