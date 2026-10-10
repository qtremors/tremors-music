// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use rodio::Source;
use std::path::Path;
use tremors_core::scanner;

#[test]
fn common_formats_decode_seek_and_scan_with_the_production_libraries() {
    let temp = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for name in [
        "tone.wav",
        "tone.mp3",
        "tone.flac",
        "tone.ogg",
        "tone-aac.m4a",
        "tone-alac.m4a",
        "tone.aac",
    ] {
        let path = root.join(name);
        let mut decoder = rodio::Decoder::try_from(std::fs::File::open(&path).unwrap())
            .unwrap_or_else(|e| panic!("{name} decoder: {e}"));
        assert!(decoder.next().is_some(), "{name}");
        decoder
            .try_seek(std::time::Duration::from_millis(100))
            .unwrap_or_else(|e| panic!("{name} seek: {e}"));
        assert!(
            decoder.take(512).any(|s| s.abs() > 0.001),
            "{name} must decode actual audio after seeking"
        );
        let song = scanner::read_song(&path, temp.path())
            .unwrap_or_else(|e| panic!("{name} metadata: {e:#}"));
        assert!(song.duration > 0.2, "{name} duration {}", song.duration);
        if name != "tone.aac" {
            assert_eq!(song.title, "Fixture Tone", "{name}");
            assert_eq!(song.artist, "Tremors Test", "{name}");
            assert_eq!(song.album, "Test Tone", "{name}");
        }
    }
}

#[test]
fn artwork_is_extracted_to_a_reusable_local_png() {
    let temp = tempfile::tempdir().unwrap();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone-art.flac");
    let first = scanner::read_song(&path, temp.path()).unwrap();
    let cover = first.cover.expect("embedded artwork should be cached");
    assert!(cover.starts_with(temp.path()));
    let image = image::open(&cover).unwrap();
    assert!(image.width() <= 512 && image.height() <= 512);
    let second = scanner::read_song(&path, temp.path()).unwrap();
    assert_eq!(second.cover, Some(cover));
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
}
