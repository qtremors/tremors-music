// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Validate edits on a sibling copy, then atomically replace the original.
use super::*;
use crate::scanner::fingerprint;
use anyhow::{Context, ensure};
use lofty::{
    config::WriteOptions,
    file::AudioFile,
    tag::{ItemValue, TagExt},
};
use std::io::{Seek, SeekFrom};

pub fn save(document: &MetadataDocument) -> Result<()> {
    let path = &document.path;
    ensure!(
        !std::fs::symlink_metadata(path)?.file_type().is_symlink(),
        "The music path became a symbolic link. Reopen the editor."
    );
    ensure!(
        !std::fs::metadata(path)?.permissions().readonly(),
        "The music file is read-only"
    );
    ensure!(
        fingerprint(path) == document.stamp,
        "This file or its lyrics changed outside the editor. Reopen it before saving."
    );
    let file = lofty::read_from_path(path)?;
    let mut tag = editing_tag(&file);
    ensure!(
        tag.tag_type() == document.tag_type,
        "Tag format changed. Reopen the editor."
    );
    for field in &document.fields {
        let original = field
            .original
            .as_ref()
            .and_then(|item| item.value().text().or_else(|| item.value().locator()))
            .unwrap_or_default();
        if field.value == original {
            continue;
        }
        ensure!(
            !field.value.contains('\0'),
            "{} contains a null character",
            field.label
        );
        if !field.value.is_empty()
            && matches!(
                field.key,
                ItemKey::TrackNumber
                    | ItemKey::TrackTotal
                    | ItemKey::DiscNumber
                    | ItemKey::DiscTotal
            )
        {
            field
                .value
                .trim()
                .parse::<u32>()
                .with_context(|| format!("{} must be a whole number", field.label))?;
        }
        if let Some(original) = &field.original {
            tag.retain(|item| item != original);
        }
        if field.value.is_empty() {
            continue;
        }
        let locator = field
            .original
            .as_ref()
            .is_some_and(|item| item.value().locator().is_some());
        let mut item = TagItem::new(
            field.key,
            if locator {
                ItemValue::Locator(field.value.clone())
            } else {
                ItemValue::Text(field.value.clone())
            },
        );
        if let Some(original) = &field.original {
            item.set_lang(*original.lang());
            item.set_description(original.description().to_owned());
        }
        ensure!(
            tag.push(item),
            "{} cannot be written in this format",
            field.label
        );
    }
    let pictures: Vec<_> = document
        .pictures
        .iter()
        .map(|a| {
            let mut picture = a.picture.clone();
            // MP4 cover atoms store image data without picture roles or descriptions.
            if document.tag_type == TagType::Mp4Ilst {
                picture.set_pic_type(PictureType::Other);
                picture.set_description(None);
            }
            picture
        })
        .collect();
    if pictures != document.original_pictures {
        while tag.picture_count() > 0 {
            tag.remove_picture(0);
        }
        for picture in &pictures {
            tag.push_picture(picture.clone());
        }
    }
    let parent = path
        .parent()
        .context("Music file has no parent directory")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    std::fs::copy(path, temporary.path())
        .context("Cannot create a working copy of the music file")?;
    tag.save_to(temporary.as_file_mut(), WriteOptions::default())
        .context("Metadata could not be written; original file was kept")?;
    temporary.as_file_mut().seek(SeekFrom::Start(0))?;
    let verified = lofty::read_from(temporary.as_file_mut())
        .context("Edited file did not pass validation; original file was kept")?;
    ensure!(
        verified.file_type() == file.file_type(),
        "Edited file changed format; original file was kept"
    );
    let written = verified
        .tag(document.tag_type)
        .context("Edited tags could not be read back")?;
    ensure!(
        verified.properties().sample_rate() == file.properties().sample_rate()
            && verified.properties().channels() == file.properties().channels(),
        "Edited audio properties changed; original file was kept"
    );
    for field in &document.fields {
        let original = field
            .original
            .as_ref()
            .and_then(|item| item.value().text().or_else(|| item.value().locator()))
            .unwrap_or_default();
        if field.value == original || field.value.is_empty() {
            continue;
        }
        let matches = written
            .items()
            .filter(|item| {
                item.key().map_key(document.tag_type) == field.key.map_key(document.tag_type)
            })
            .filter_map(|item| item.value().text().or_else(|| item.value().locator()))
            .any(|value| {
                value == field.value
                    || (field.key.is_numeric()
                        && value
                            .parse::<u64>()
                            .ok()
                            .zip(field.value.parse::<u64>().ok())
                            .is_some_and(|(a, b)| a == b))
            });
        ensure!(
            matches,
            "{} did not round trip in this file format; original file was kept",
            field.label
        );
    }
    if pictures != document.original_pictures {
        ensure!(
            written.pictures().len() == pictures.len()
                && written
                    .pictures()
                    .iter()
                    .zip(&pictures)
                    .all(|(a, b)| a.data() == b.data()
                        && a.pic_type() == b.pic_type()
                        && a.description().unwrap_or_default()
                            == b.description().unwrap_or_default()),
            "Artwork could not be stored by this format; original file was kept"
        );
    }
    ensure!(
        fingerprint(path) == document.stamp,
        "File changed while saving; original file was kept"
    );
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|e| e.error).context("Cannot replace the music file. Stop playback and check folder permissions; original file was kept")?;
    Ok(())
}
