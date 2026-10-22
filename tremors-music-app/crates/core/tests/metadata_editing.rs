// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use lofty::tag::ItemKey;
use std::path::{Path, PathBuf};
use tremors_core::{metadata, scanner};

fn fixture(name: &str, root: &Path) -> PathBuf {
    let path = root.join(name);
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
        &path,
    )
    .unwrap();
    path
}
fn samples(path: &Path) -> Vec<f32> {
    rodio::Decoder::try_from(std::fs::File::open(path).unwrap())
        .unwrap()
        .collect()
}
fn change(doc: &mut metadata::MetadataDocument, key: ItemKey, value: &str) {
    if !doc.fields.iter().any(|f| f.key == key) {
        doc.add_field(key)
            .unwrap_or_else(|e| panic!("{:?} {key:?}: {e}", doc.path));
    }
    doc.fields.iter_mut().find(|f| f.key == key).unwrap().value = value.into();
}

#[test]
fn native_tags_round_trip_without_changing_audio_in_every_supported_container() {
    let temp = tempfile::tempdir().unwrap();
    for name in [
        "tone.wav",
        "tone.mp3",
        "tone.flac",
        "tone.ogg",
        "tone-aac.m4a",
        "tone-alac.m4a",
        "tone.aac",
    ] {
        let path = fixture(name, temp.path());
        let before = samples(&path);
        let mut doc = metadata::read(7, &path, &temp.path().join("covers")).unwrap();
        change(&mut doc, ItemKey::TrackTitle, "Edited 世界");
        change(&mut doc, ItemKey::TrackArtist, "New artist");
        change(&mut doc, ItemKey::AlbumArtist, "Album artist");
        change(&mut doc, ItemKey::TrackNumber, "3");
        change(&mut doc, ItemKey::Composer, "Composer test");
        let lyrics = if doc.tag_type == lofty::tag::TagType::Id3v2 {
            ItemKey::UnsyncLyrics
        } else {
            ItemKey::Lyrics
        };
        change(&mut doc, lyrics, "First line\nSecond line");
        metadata::save(&doc).unwrap_or_else(|e| panic!("{name}: {e:#}"));
        let saved = metadata::read(7, &path, &temp.path().join("covers")).unwrap();
        for key in [
            ItemKey::TrackTitle,
            ItemKey::TrackArtist,
            ItemKey::AlbumArtist,
            ItemKey::TrackNumber,
            ItemKey::Composer,
            lyrics,
        ] {
            assert_eq!(
                saved.fields.iter().find(|f| f.key == key).unwrap().value,
                doc.fields.iter().find(|f| f.key == key).unwrap().value,
                "{name}: {key:?}"
            );
        }
        assert_eq!(samples(&path), before, "Audio samples changed in {name}");
        let song = scanner::read_song(&path, &temp.path().join("covers")).unwrap();
        assert_eq!(
            song.title, "Edited 世界",
            "Scanner must see edited title in {name}"
        );
        assert_eq!(song.details.lyrics, "First line\nSecond line", "{name}");
    }
}

#[test]
fn embedded_pictures_can_be_added_replaced_and_removed_without_changing_audio() {
    let temp = tempfile::tempdir().unwrap();
    let cover = temp.path().join("cover.png");
    image::RgbImage::from_pixel(32, 32, image::Rgb([40, 80, 120]))
        .save(&cover)
        .unwrap();
    for name in [
        "tone.wav",
        "tone.mp3",
        "tone.flac",
        "tone.ogg",
        "tone-aac.m4a",
        "tone-alac.m4a",
    ] {
        let path = fixture(name, temp.path());
        let before = samples(&path);
        let mut doc = metadata::read(1, &path, &temp.path().join("covers")).unwrap();
        doc.pictures
            .push(metadata::load_artwork(&cover, &temp.path().join("covers")).unwrap());
        metadata::save(&doc).unwrap_or_else(|e| panic!("{name}: {e:#}"));
        let mut doc = metadata::read(1, &path, &temp.path().join("covers")).unwrap();
        assert_eq!(doc.pictures.len(), 1, "{name}");
        doc.pictures.clear();
        metadata::save(&doc).unwrap();
        assert!(
            metadata::read(1, &path, &temp.path().join("covers"))
                .unwrap()
                .pictures
                .is_empty()
        );
        assert_eq!(samples(&path), before, "{name}");
    }
}

#[test]
fn invalid_fields_and_outside_modifications_keep_original_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let path = fixture("tone.mp3", temp.path());
    let before = std::fs::read(&path).unwrap();
    let mut doc = metadata::read(1, &path, temp.path()).unwrap();
    change(&mut doc, ItemKey::TrackNumber, "not a number");
    assert!(metadata::save(&doc).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    change(&mut doc, ItemKey::TrackNumber, "2");
    std::fs::write(path.with_extension("lrc"), "[00:01]Outside change").unwrap();
    assert!(
        metadata::save(&doc)
            .unwrap_err()
            .to_string()
            .contains("outside")
    );
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[test]
fn editing_one_field_keeps_other_comments_and_private_frames() {
    use lofty::{
        config::WriteOptions,
        file::TaggedFileExt,
        id3::v2::{BinaryFrame, Frame, FrameId, Id3v2Tag},
        tag::{ItemValue, Tag, TagExt, TagItem},
    };
    let temp = tempfile::tempdir().unwrap();
    let path = fixture("tone.mp3", temp.path());
    let mut native = Id3v2Tag::default();
    native.insert(Frame::Binary(BinaryFrame::new(
        FrameId::Valid("PRIV".into()),
        b"tremors.fixture\0private-data".to_vec(),
    )));
    let mut tag: Tag = native.into();
    for (description, value) in [("first", "Keep this"), ("second", "Edit this")] {
        let mut item = TagItem::new(ItemKey::Comment, ItemValue::Text(value.into()));
        item.set_description(description.into());
        item.set_lang(*b"eng");
        assert!(tag.push(item));
    }
    tag.save_to_path(&path, WriteOptions::default()).unwrap();
    let mut doc = metadata::read(1, &path, temp.path()).unwrap();
    let comment = doc
        .fields
        .iter_mut()
        .find(|f| f.value == "Edit this")
        .unwrap();
    comment.value = "Updated comment".into();
    change(&mut doc, ItemKey::TrackTitle, "Title change");
    metadata::save(&doc).unwrap();
    let file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag().unwrap();
    assert!(tag.items().any(|i| i.key() == ItemKey::Comment
        && i.description() == "first"
        && i.value().text() == Some("Keep this")));
    assert!(tag.items().any(|i| i.key() == ItemKey::Comment
        && i.description() == "second"
        && i.value().text() == Some("Updated comment")));
    let native: Id3v2Tag = tag.clone().into();
    let Some(Frame::Private(frame)) = native.get(&FrameId::Valid("PRIV".into())) else {
        panic!("Private frame lost")
    };
    assert_eq!(frame.owner, "tremors.fixture");
    assert_eq!(frame.private_data.as_ref(), b"private-data");
}

#[test]
fn local_webp_artwork_is_converted_to_a_native_png_picture() {
    let temp = tempfile::tempdir().unwrap();
    let image = temp.path().join("image.webp");
    image::RgbImage::from_pixel(16, 16, image::Rgb([10, 50, 90]))
        .save(&image)
        .unwrap();
    let artwork = metadata::load_artwork(&image, &temp.path().join("covers")).unwrap();
    assert!(artwork.picture.data().starts_with(b"\x89PNG"));
    assert!(artwork.preview.unwrap().is_file());
}
