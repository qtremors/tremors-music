// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use tremors_core::{model::Song, queue::Queue, settings::RepeatMode};
fn song(id: i64) -> Song {
    Song {
        id,
        title: id.to_string(),
        ..Default::default()
    }
}

#[test]
fn repeat_one_only_repeats_on_automatic_completion() {
    let mut q = Queue::default();
    q.replace(vec![song(1), song(2)], 0);
    q.repeat = RepeatMode::One;
    assert_eq!(q.next(true).unwrap().id, 1);
    assert_eq!(q.next(false).unwrap().id, 2);
    assert_eq!(q.next(true).unwrap().id, 2);
    assert!(q.next(false).is_none());
}
#[test]
fn repeat_all_wraps_and_off_stops_at_queue_end() {
    let mut q = Queue::default();
    q.replace(vec![song(1), song(2)], 1);
    assert!(q.next(true).is_none());
    q.repeat = RepeatMode::All;
    assert_eq!(q.next(true).unwrap().id, 1);
    assert_eq!(q.previous().unwrap().id, 2);
}
#[test]
fn duplicate_song_ids_have_distinct_queue_positions() {
    let mut q = Queue::default();
    q.replace(vec![song(1), song(2), song(1)], 2);
    assert_eq!(q.current_index(), Some(2));
    q.set_shuffle(true);
    assert_eq!(q.current_index(), Some(0));
    q.set_shuffle(false);
    assert_eq!(q.current_index(), Some(2));
    assert_eq!(q.current().unwrap().id, 1);
}
#[test]
fn shuffle_preserves_current_and_restores_original_order() {
    let mut q = Queue::default();
    q.replace((0..30).map(song).collect(), 17);
    q.set_shuffle(true);
    assert_eq!(q.current().unwrap().id, 17);
    let mut shuffled = q.ordered().iter().map(|s| s.id).collect::<Vec<_>>();
    shuffled.sort();
    assert_eq!(shuffled, (0..30).collect::<Vec<_>>());
    q.set_shuffle(false);
    assert_eq!(q.current_index(), Some(17));
    assert_eq!(
        q.ordered().iter().map(|s| s.id).collect::<Vec<_>>(),
        (0..30).collect::<Vec<_>>()
    );
}
#[test]
fn removing_before_current_keeps_current_track_and_removing_current_selects_next() {
    let mut q = Queue::default();
    q.replace(vec![song(1), song(2), song(3)], 1);
    assert!(!q.remove(0));
    assert_eq!(q.current().unwrap().id, 2);
    assert_eq!(q.current_index(), Some(0));
    assert!(q.remove(0));
    assert_eq!(q.current().unwrap().id, 3);
    assert!(q.remove(0));
    assert!(q.current().is_none());
    assert!(q.is_empty());
}
#[test]
fn empty_and_invalid_queue_operations_are_safe() {
    let mut q = Queue::default();
    assert!(q.next(true).is_none());
    assert!(q.previous().is_none());
    q.set_shuffle(true);
    q.replace(vec![], 0);
    assert!(!q.remove(0));
    assert!(q.select(999).is_none());
    q.append(song(1));
    assert_eq!(q.next(false).unwrap().id, 1);
    q.clear();
    assert!(q.current().is_none());
}
