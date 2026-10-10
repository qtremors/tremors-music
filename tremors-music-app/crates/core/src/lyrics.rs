// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LyricLine {
    pub seconds: f64,
    pub text: String,
}

/// Parse standard LRC, including multiple timestamps and signed millisecond offsets.
/// Plain lyrics stay plain; malformed timecodes do not become seek targets.
pub fn parse_lrc(text: &str) -> Vec<LyricLine> {
    let offset = text
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("[offset:")
                .and_then(|s| s.strip_suffix(']'))
                .and_then(|s| s.parse::<i64>().ok())
        })
        .unwrap_or(0) as f64
        / 1000.;
    let mut lines = Vec::new();
    for line in text.lines() {
        let mut rest = line.trim();
        let mut stamps = Vec::new();
        while let Some(s) = rest.strip_prefix('[') {
            let Some((stamp, tail)) = s.split_once(']') else {
                break;
            };
            let Some((minutes, seconds)) = stamp.split_once(':') else {
                break;
            };
            let (Ok(minutes), Ok(seconds)) = (minutes.parse::<u32>(), seconds.parse::<f64>())
            else {
                break;
            };
            if !seconds.is_finite() || !(0. ..60.).contains(&seconds) {
                break;
            }
            stamps.push((minutes as f64 * 60. + seconds + offset).max(0.));
            rest = tail;
        }
        for seconds in stamps {
            lines.push(LyricLine {
                seconds,
                text: rest.trim().to_owned(),
            });
        }
    }
    lines.sort_by(|a, b| a.seconds.total_cmp(&b.seconds));
    lines
}

pub fn active_line(lines: &[LyricLine], position: f64) -> Option<usize> {
    lines
        .partition_point(|line| line.seconds <= position)
        .checked_sub(1)
}
