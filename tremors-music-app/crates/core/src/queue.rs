// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct QueueState {
    pub song_ids: Vec<i64>,
    pub order: Vec<usize>,
    pub cursor: Option<usize>,
    pub position: f64,
}

