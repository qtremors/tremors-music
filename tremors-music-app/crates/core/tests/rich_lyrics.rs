// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors
use tremors_core::{
    lyrics::{self, parse_lrc},
    model::Song,
    scanner,
};
#[test]
fn enhanced_lrc_keeps_unicode_word_ranges_and_translations_together() {
    let lines = parse_lrc(
        "[offset:100]\n[00:01.00]<00:01.00>Héllo <00:01.50>世界\n[00:01.00]Translation\n[00:03.00]Next",
    );
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].text, "Héllo 世界\nTranslation");
    assert_eq!(lines[0].seconds, 1.1);
    let words = &lines[0].words;
    assert_eq!(words.len(), 2);
    assert_eq!(&lines[0].text[words[0].start..words[0].end], "Héllo ");
    assert_eq!(&lines[0].text[words[1].start..words[1].end], "世界");
    assert_eq!(words[1].seconds, 1.6);
    assert!(
        parse_lrc("[00:01]plain <not a timestamp>")[0]
            .words
            .is_empty()
    );
}
#[test]
fn local_ttml_parses_nested_timing_entities_and_word_spans() {
    let xml = r#"<tt xmlns="http://www.w3.org/ns/ttml"><body begin="1s"><div><p begin="00:00:02.000"><span begin="0s">Hello &amp; </span><span begin="500ms">world</span><br/>Translation</p><p begin="4s">Next</p></div></body></tt>"#;
    let lines = parse_lrc(xml);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].seconds, 3.);
    assert_eq!(lines[0].text, "Hello & world\nTranslation");
    assert_eq!(lines[0].words[1].seconds, 3.5);
    assert_eq!(lines[1].seconds, 5.);
    assert!(parse_lrc("<tt><p begin=\"oops\">Bad</p></tt>").is_empty());
    assert!(parse_lrc("<!DOCTYPE tt [<!ENTITY x SYSTEM 'https://example.invalid'>]><tt><p begin=\"1s\">&x;</p></tt>").is_empty());
}
#[test]
fn lyric_save_is_atomic_detects_external_changes_and_never_touches_audio() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("song.mp3");
    std::fs::write(&path, b"audio stays unchanged").unwrap();
    let song = Song {
        path: path.clone(),
        ..Default::default()
    };
    let mut document = lyrics::file::read(&song).unwrap();
    document.text = "[00:01]New line".into();
    lyrics::file::save(&document).unwrap();
    assert_eq!(
        std::fs::read_to_string(path.with_extension("lrc")).unwrap(),
        document.text
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"audio stays unchanged");
    assert!(lyrics::file::save(&document).is_err());
    let mut document = lyrics::file::read(&song).unwrap();
    std::fs::write(path.with_extension("lrc"), "Outside change").unwrap();
    document.text = "Overwrite".into();
    assert!(lyrics::file::save(&document).is_err());
    assert_eq!(
        std::fs::read_to_string(path.with_extension("lrc")).unwrap(),
        "Outside change"
    );
}
#[test]
fn richer_sidecars_invalidate_the_incremental_scan_fingerprint() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("song.mp3");
    std::fs::write(&path, b"audio").unwrap();
    let original = scanner::fingerprint(&path);
    std::fs::write(path.with_extension("ttml"), "<tt/>").unwrap();
    let ttml = scanner::fingerprint(&path);
    assert_ne!(original.2, ttml.2);
    std::fs::write(path.with_extension("txt"), "plain lyrics").unwrap();
    assert_ne!(ttml.2, scanner::fingerprint(&path).2);
}
