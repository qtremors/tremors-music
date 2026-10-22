// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

//! Read native tag fields and decode artwork for the editor.
use super::*;
use crate::scanner::{self, fingerprint};
use anyhow::{Context, ensure};
use std::{io::Read, path::Path};

pub fn read(song_id: i64, path: &Path, covers: &Path) -> Result<MetadataDocument> {
    ensure!(
        !std::fs::symlink_metadata(path)?.file_type().is_symlink(),
        "Edit the original music file rather than a symbolic link"
    );
    let before = fingerprint(path);
    let file = lofty::read_from_path(path).context("Cannot open metadata")?;
    let tag = editing_tag(&file);
    let mut fields = Vec::new();
    for (name, _) in COMMON {
        let Some(key) = ItemKey::VARIANTS
            .iter()
            .copied()
            .find(|k| format!("{k:?}") == *name)
        else {
            continue;
        };
        if key.map_key(tag.tag_type()).is_none()
            || fields.iter().any(|field: &MetadataField| {
                field.key.map_key(tag.tag_type()) == key.map_key(tag.tag_type())
            })
        {
            continue;
        }
        let items: Vec<_> = tag
            .items()
            .filter(|item| {
                item.key() == key
                    && item
                        .value()
                        .text()
                        .or_else(|| item.value().locator())
                        .is_some()
            })
            .cloned()
            .collect();
        if items.is_empty() {
            fields.push(MetadataField {
                key,
                label: label(key),
                value: String::new(),
                original: None,
            });
        } else {
            for item in items {
                fields.push(field(item));
            }
        }
    }
    for item in tag.items() {
        if item
            .value()
            .text()
            .or_else(|| item.value().locator())
            .is_some()
            && !fields.iter().any(|f| f.original.as_ref() == Some(item))
        {
            fields.push(field(item.clone()));
        }
    }
    let pictures = tag
        .pictures()
        .iter()
        .map(|picture| {
            Ok(Artwork {
                preview: scanner::cache_cover(picture.data(), covers).ok(),
                picture: picture.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        fingerprint(path) == before,
        "File changed while loading metadata. Reopen the editor."
    );
    Ok(MetadataDocument {
        song_id,
        path: path.into(),
        fields,
        pictures,
        tag_type: tag.tag_type(),
        original_pictures: tag.pictures().to_vec(),
        stamp: before,
        has_sidecar_lyrics: ["lrc", "ttml", "txt"]
            .into_iter()
            .any(|ext| path.with_extension(ext).is_file()),
    })
}

fn field(item: TagItem) -> MetadataField {
    let value = item
        .value()
        .text()
        .or_else(|| item.value().locator())
        .unwrap_or_default()
        .to_owned();
    let mut name = label(item.key());
    if !item.description().is_empty() {
        name.push_str(&format!(" · {}", item.description()));
    }
    MetadataField {
        key: item.key(),
        label: name,
        value,
        original: Some(item),
    }
}

pub fn load_artwork(path: &Path, covers: &Path) -> Result<Artwork> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(20 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= 20 * 1024 * 1024,
        "Artwork must be 20 MiB or smaller"
    );
    let preview = scanner::cache_cover(&bytes, covers)?;
    let mut picture = match Picture::from_reader(&mut std::io::Cursor::new(&bytes)) {
        Ok(picture) => picture,
        Err(_) => {
            // Convert local images unsupported by native picture tags to lossless PNG.
            let image = image::load_from_memory(&bytes)?;
            let mut png = std::io::Cursor::new(Vec::new());
            image.write_to(&mut png, image::ImageFormat::Png)?;
            ensure!(
                png.get_ref().len() <= 20 * 1024 * 1024,
                "Converted artwork exceeds 20 MiB"
            );
            png.set_position(0);
            Picture::from_reader(&mut png)?
        }
    };
    picture.set_pic_type(PictureType::CoverFront);
    Ok(Artwork {
        picture,
        preview: Some(preview),
    })
}
