// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use serde::{Deserialize, Serialize};
pub mod file;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LyricLine {
    pub seconds: f64,
    pub text: String,
    #[serde(default)]
    pub words: Vec<LyricWord>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LyricWord {
    pub seconds: f64,
    pub start: usize,
    pub end: usize,
}

/// Parse standard LRC, including multiple timestamps and signed millisecond offsets.
/// Plain lyrics stay plain; malformed timecodes do not become seek targets.
pub fn parse_lrc(text: &str) -> Vec<LyricLine> {
    if text.trim_start().starts_with('<') {
        return parse_ttml(text);
    }
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
        let (clean, words) = enhanced_text(rest.trim(), offset);
        for seconds in stamps {
            lines.push(LyricLine {
                seconds,
                text: clean.clone(),
                words: words.clone(),
            });
        }
    }
    lines.sort_by(|a, b| a.seconds.total_cmp(&b.seconds));
    // Multiple languages at the same line time stay together rather than fighting for focus.
    let mut merged: Vec<LyricLine> = Vec::new();
    for mut line in lines {
        if let Some(previous) = merged.last_mut().filter(|p| p.seconds == line.seconds) {
            let shift = previous.text.len() + 1;
            previous.text.push('\n');
            previous.text.push_str(&line.text);
            for word in &mut line.words {
                word.start += shift;
                word.end += shift;
            }
            previous.words.extend(line.words);
        } else {
            merged.push(line);
        }
    }
    merged
}

pub fn active_line(lines: &[LyricLine], position: f64) -> Option<usize> {
    lines
        .partition_point(|line| line.seconds <= position)
        .checked_sub(1)
}

fn clock(text: &str) -> Option<f64> {
    let seconds = if let Some(ms) = text.strip_suffix("ms") {
        ms.parse::<f64>().ok()? / 1000.
    } else if let Some(s) = text.strip_suffix('s') {
        s.parse().ok()?
    } else {
        let parts: Vec<_> = text.split(':').collect();
        match parts.as_slice() {
            [m, s] => m.parse::<u32>().ok()? as f64 * 60. + s.parse::<f64>().ok()?,
            [h, m, s] => {
                h.parse::<u32>().ok()? as f64 * 3600.
                    + m.parse::<u32>().ok()? as f64 * 60.
                    + s.parse::<f64>().ok()?
            }
            _ => return None,
        }
    };
    (seconds.is_finite() && seconds >= 0.).then_some(seconds)
}

fn enhanced_text(text: &str, offset: f64) -> (String, Vec<LyricWord>) {
    let mut output = String::new();
    let mut words: Vec<LyricWord> = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('<') {
        output.push_str(&rest[..open]);
        if let Some(previous) = words.last_mut() {
            previous.end = output.len();
        }
        let Some(close) = rest[open..].find('>').map(|i| i + open) else {
            output.push_str(&rest[open..]);
            rest = "";
            break;
        };
        let Some(seconds) = clock(&rest[open + 1..close]) else {
            output.push_str(&rest[open..=close]);
            rest = &rest[close + 1..];
            continue;
        };
        words.push(LyricWord {
            seconds: (seconds + offset).max(0.),
            start: output.len(),
            end: output.len(),
        });
        rest = &rest[close + 1..];
    }
    output.push_str(rest);
    if let Some(previous) = words.last_mut() {
        previous.end = output.len();
    }
    words.retain(|w| w.end > w.start);
    (output, words)
}

/// Local TTML clock-time paragraphs and spans; XML entities are decoded by the parser.
/// External entities, network references, styling, and scripts are never evaluated.
pub fn parse_ttml(text: &str) -> Vec<LyricLine> {
    let Ok(document) = roxmltree::Document::parse(text) else {
        return vec![];
    };
    let mut lines = Vec::new();
    for paragraph in document.descendants().filter(|n| n.has_tag_name("p")) {
        let Some(begin) = paragraph.attribute("begin").and_then(clock) else {
            continue;
        };
        let parent_offset: f64 = paragraph
            .ancestors()
            .skip(1)
            .filter_map(|n| n.attribute("begin").and_then(clock))
            .sum();
        let mut line = LyricLine {
            seconds: begin + parent_offset,
            text: String::new(),
            words: vec![],
        };
        for node in paragraph.descendants() {
            if node.is_element()
                && node
                    .attributes()
                    .any(|a| a.name() == "role" && a.value().contains("translation"))
                && !line.text.ends_with('\n')
                && !line.text.is_empty()
            {
                line.text.push('\n');
            }
            if node.is_element() && node.has_tag_name("br") {
                line.text.push('\n');
            }
            if !node.is_text() {
                continue;
            }
            let text = node.text().unwrap_or_default();
            let start = line.text.len();
            line.text.push_str(text);
            if let Some(span) = node
                .ancestors()
                .skip(1)
                .take_while(|n| n.id() != paragraph.id())
                .find(|n| n.has_tag_name("span") && n.attribute("begin").is_some())
                && let Some(at) = span.attribute("begin").and_then(clock)
            {
                line.words.push(LyricWord {
                    seconds: line.seconds + at,
                    start,
                    end: line.text.len(),
                });
            }
        }
        if !line.text.trim().is_empty() {
            lines.push(line);
        }
    }
    lines.sort_by(|a, b| a.seconds.total_cmp(&b.seconds));
    lines
}
