// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use crate::{model::Song, settings::RepeatMode};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct QueueState {
    pub song_ids: Vec<i64>,
    pub order: Vec<usize>,
    pub cursor: Option<usize>,
    pub position: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Queue {
    tracks: Vec<Song>,
    order: Vec<usize>,
    cursor: Option<usize>,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

impl Queue {
    pub fn replace(&mut self, tracks: Vec<Song>, selected: usize) {
        self.tracks = tracks;
        self.order = (0..self.tracks.len()).collect();
        self.cursor = if selected < self.tracks.len() {
            Some(selected)
        } else {
            None
        };
        if self.shuffle {
            self.reshuffle();
        }
    }

    pub fn current(&self) -> Option<&Song> {
        self.cursor
            .and_then(|i| self.order.get(i))
            .and_then(|i| self.tracks.get(*i))
    }

    pub fn current_index(&self) -> Option<usize> {
        self.cursor
    }
    pub fn len(&self) -> usize {
        self.order.len()
    }
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
    pub fn ordered(&self) -> Vec<Song> {
        self.order.iter().map(|i| self.tracks[*i].clone()).collect()
    }

    pub fn select(&mut self, index: usize) -> Option<Song> {
        if index >= self.order.len() {
            return None;
        }
        self.cursor = Some(index);
        self.current().cloned()
    }

    pub fn next(&mut self, automatic: bool) -> Option<Song> {
        if self.is_empty() {
            return None;
        }
        if automatic && self.repeat == RepeatMode::One {
            return self.current().cloned();
        }
        let next = self.cursor.map_or(0, |i| i + 1);
        if next < self.len() {
            self.cursor = Some(next);
        } else if self.repeat == RepeatMode::All {
            self.cursor = Some(0);
        } else {
            return None;
        }
        self.current().cloned()
    }

    pub fn previous(&mut self) -> Option<Song> {
        if self.is_empty() {
            return None;
        }
        let current = self.cursor.unwrap_or(0);
        self.cursor = Some(if current > 0 {
            current - 1
        } else if self.repeat == RepeatMode::All {
            self.len() - 1
        } else {
            0
        });
        self.current().cloned()
    }

    pub fn set_shuffle(&mut self, shuffle: bool) {
        if self.shuffle == shuffle {
            return;
        }
        let current = self.cursor.map(|i| self.order[i]);
        self.shuffle = shuffle;
        if shuffle {
            self.reshuffle();
        } else {
            self.order = (0..self.tracks.len()).collect();
            self.cursor = current;
        }
    }

    fn reshuffle(&mut self) {
        let current = self.cursor.map(|i| self.order[i]);
        self.order.shuffle(&mut rand::rng());
        if let Some(current) = current {
            if let Some(index) = self.order.iter().position(|i| *i == current) {
                self.order.swap(0, index);
            }
            self.cursor = Some(0);
        }
    }

    pub fn append(&mut self, song: Song) {
        self.order.push(self.tracks.len());
        self.tracks.push(song);
    }

    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.len() {
            return false;
        }
        let current_removed = self.cursor == Some(index);
        let source = self.order.remove(index);
        self.tracks.remove(source);
        for item in &mut self.order {
            if *item > source {
                *item -= 1;
            }
        }
        self.cursor = match self.cursor {
            _ if self.is_empty() => None,
            Some(cursor) if cursor > index => Some(cursor - 1),
            Some(cursor) => Some(cursor.min(self.len() - 1)),
            None => None,
        };
        current_removed
    }

    pub fn play_next(&mut self, song: Song) {
        let source = self
            .cursor
            .map(|i| self.order[i] + 1)
            .unwrap_or(self.tracks.len());
        self.tracks.insert(source, song);
        for item in &mut self.order {
            if *item >= source {
                *item += 1;
            }
        }
        let target = self.cursor.map_or(self.order.len(), |i| i + 1);
        self.order.insert(target, source);
    }

    pub fn move_to(&mut self, from: usize, to: usize) {
        if from >= self.len() || to >= self.len() || from == to {
            return;
        }
        let current = self.cursor.map(|i| self.order[i]);
        let source = self.order.remove(from);
        self.order.insert(to, source);
        self.cursor = current.and_then(|i| self.order.iter().position(|v| *v == i));
        if !self.shuffle {
            let current = self.cursor;
            self.tracks = self.ordered();
            self.order = (0..self.tracks.len()).collect();
            self.cursor = current;
        }
    }

    pub fn state(&self, position: f64) -> QueueState {
        QueueState {
            song_ids: self.tracks.iter().map(|s| s.id).collect(),
            order: self.order.clone(),
            cursor: self.cursor,
            position: if position.is_finite() {
                position.max(0.)
            } else {
                0.
            },
        }
    }

    pub fn restore(&mut self, saved: &QueueState, songs: &[Song]) -> f64 {
        let by_id: std::collections::HashMap<_, _> = songs.iter().map(|s| (s.id, s)).collect();
        let mut remap = vec![None; saved.song_ids.len()];
        self.tracks.clear();
        for (index, id) in saved.song_ids.iter().enumerate() {
            if let Some(song) = by_id.get(id) {
                remap[index] = Some(self.tracks.len());
                self.tracks.push((**song).clone());
            }
        }
        let mut valid = saved.order.clone();
        valid.sort_unstable();
        let order = if valid == (0..saved.song_ids.len()).collect::<Vec<_>>() {
            saved.order.clone()
        } else {
            (0..saved.song_ids.len()).collect()
        };
        let current_source = saved.cursor.and_then(|i| order.get(i)).copied();
        self.order = order.iter().filter_map(|i| remap[*i]).collect();
        self.cursor = current_source
            .and_then(|i| remap[i])
            .and_then(|source| self.order.iter().position(|i| *i == source));
        if self.cursor.is_some() && saved.position.is_finite() {
            saved
                .position
                .max(0.)
                .min(self.current().map_or(0., |s| s.duration.max(0.)))
        } else {
            0.
        }
    }

    pub fn reconcile(&mut self, songs: &[Song]) -> bool {
        let by_id: std::collections::HashMap<_, _> = songs.iter().map(|s| (s.id, s)).collect();
        let missing: Vec<_> = self
            .order
            .iter()
            .enumerate()
            .filter_map(|(index, source)| {
                (!by_id.contains_key(&self.tracks[*source].id)).then_some(index)
            })
            .collect();
        let mut current_removed = false;
        for index in missing.into_iter().rev() {
            current_removed |= self.remove(index);
        }
        for song in &mut self.tracks {
            if let Some(updated) = by_id.get(&song.id)
                && song != *updated
            {
                *song = (*updated).clone();
            }
        }
        current_removed
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.order.clear();
        self.cursor = None;
    }
}
