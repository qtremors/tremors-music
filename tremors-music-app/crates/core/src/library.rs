// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2025-2026 Tremors and contributors

use crate::{
    model::{Playlist, PlaylistEntry, Snapshot, Song, now},
    queue::QueueState,
    settings::Preferences,
};
use anyhow::{Context, Result, bail};
use rusqlite::{Connection, params};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::Duration,
};

pub struct Library {
    connection: Connection,
}

const SONG_COLUMNS: &str = "id,path,title,artist,album,album_artist,genre,year,track,disc,duration,cover,favorite,play_count,added_at,last_played,metadata";

fn song_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Song> {
    Ok(Song {
        id: row.get(0)?,
        path: PathBuf::from(row.get::<_, String>(1)?),
        title: row.get(2)?,
        artist: row.get(3)?,
        album: row.get(4)?,
        album_artist: row.get(5)?,
        genre: row.get(6)?,
        year: row.get(7)?,
        track: row.get(8)?,
        disc: row.get(9)?,
        duration: row.get(10)?,
        cover: row.get::<_, Option<String>>(11)?.map(PathBuf::from),
        favorite: row.get(12)?,
        play_count: row.get(13)?,
        added_at: row.get(14)?,
        last_played: row.get(15)?,
        details: serde_json::from_str(&row.get::<_, String>(16)?).unwrap_or_default(),
    })
}

impl Library {
    pub fn open(path: &Path) -> Result<Self> {
        let connection =
            Connection::open(path).with_context(|| format!("Cannot open {}", path.display()))?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS songs (
                id INTEGER PRIMARY KEY, path TEXT NOT NULL UNIQUE, title TEXT NOT NULL,
                artist TEXT NOT NULL, album TEXT NOT NULL, album_artist TEXT NOT NULL,
                genre TEXT NOT NULL, year INTEGER, track INTEGER, disc INTEGER,
                duration REAL NOT NULL, cover TEXT, favorite INTEGER NOT NULL DEFAULT 0,
                play_count INTEGER NOT NULL DEFAULT 0, added_at INTEGER NOT NULL, last_played INTEGER
            );
            CREATE INDEX IF NOT EXISTS songs_artist ON songs(artist);
            CREATE INDEX IF NOT EXISTS songs_album ON songs(album_artist, album);
            CREATE TABLE IF NOT EXISTS roots(path TEXT PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS playlists(id INTEGER PRIMARY KEY, name TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS playlist_entries(
                id INTEGER PRIMARY KEY,
                playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
                song_id INTEGER NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
                position INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS playlist_order ON playlist_entries(playlist_id,position);
            CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            PRAGMA user_version=3;")?;
        let columns = connection
            .prepare("PRAGMA table_info(songs)")?
            .query_map([], |r| r.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if !columns.iter().any(|name| name == "metadata") {
            connection.execute(
                "ALTER TABLE songs ADD COLUMN metadata TEXT NOT NULL DEFAULT '{}'",
                [],
            )?;
        }
        Ok(Self { connection })
    }

    pub fn snapshot(&self) -> Result<Snapshot> {
        let songs = self.connection.prepare(&format!("SELECT {SONG_COLUMNS} FROM songs ORDER BY title COLLATE NOCASE,artist COLLATE NOCASE,id"))?
            .query_map([], song_from_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
        let roots = self
            .connection
            .prepare("SELECT path FROM roots ORDER BY path")?
            .query_map([], |r| Ok(PathBuf::from(r.get::<_, String>(0)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let mut playlists = self
            .connection
            .prepare("SELECT id,name FROM playlists ORDER BY name COLLATE NOCASE,id")?
            .query_map([], |r| {
                Ok(Playlist {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    entries: Vec::new(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for pl in &mut playlists {
            pl.entries = self.connection.prepare("SELECT id,song_id FROM playlist_entries WHERE playlist_id=? ORDER BY position,id")?
                .query_map([pl.id], |r| Ok(PlaylistEntry { id:r.get(0)?, song_id:r.get(1)? }))?.collect::<rusqlite::Result<Vec<_>>>()?;
        }
        let preferences = self
            .connection
            .query_row(
                "SELECT value FROM settings WHERE key='preferences'",
                [],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Ok(Snapshot {
            songs,
            roots,
            playlists,
            preferences,
            session: self
                .connection
                .query_row("SELECT value FROM settings WHERE key='session'", [], |r| {
                    r.get::<_, String>(0)
                })
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
        })
    }

    pub fn watch_config(&self) -> Result<(Vec<PathBuf>, bool)> {
        let roots = self
            .connection
            .prepare("SELECT path FROM roots ORDER BY path")?
            .query_map([], |r| Ok(PathBuf::from(r.get::<_, String>(0)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let preferences: Preferences = self
            .connection
            .query_row(
                "SELECT value FROM settings WHERE key='preferences'",
                [],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Ok((roots, preferences.watch_folders))
    }

    pub fn add_root(&mut self, root: &Path) -> Result<()> {
        if !root.is_dir() {
            bail!("Music folder does not exist: {}", root.display());
        }
        let root = dunce::canonicalize(root)?;
        self.connection.execute(
            "INSERT OR IGNORE INTO roots(path) VALUES(?)",
            [root.to_string_lossy().as_ref()],
        )?;
        Ok(())
    }

    pub fn remove_root(&mut self, root: &Path) -> Result<()> {
        let snapshot = self.snapshot()?;
        let remaining: Vec<_> = snapshot
            .roots
            .iter()
            .filter(|p| p.as_path() != root)
            .collect();
        let tx = self.connection.transaction()?;
        tx.execute(
            "DELETE FROM roots WHERE path=?",
            [root.to_string_lossy().as_ref()],
        )?;
        for song in &snapshot.songs {
            if song.path.starts_with(root) && !remaining.iter().any(|p| song.path.starts_with(p)) {
                tx.execute("DELETE FROM songs WHERE id=?", [song.id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn apply_scan(
        &mut self,
        root: &Path,
        songs: &[Song],
        seen: &HashSet<PathBuf>,
        prune: bool,
    ) -> Result<()> {
        let old = if prune {
            self.snapshot()?.songs
        } else {
            Vec::new()
        };
        let tx = self.connection.transaction()?;
        for s in songs {
            tx.execute("INSERT INTO songs(path,title,artist,album,album_artist,genre,year,track,disc,duration,cover,added_at,metadata)
                VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
                ON CONFLICT(path) DO UPDATE SET title=excluded.title,artist=excluded.artist,album=excluded.album,
                album_artist=excluded.album_artist,genre=excluded.genre,year=excluded.year,track=excluded.track,
                disc=excluded.disc,duration=excluded.duration,cover=excluded.cover,metadata=excluded.metadata",
                params![s.path.to_string_lossy(),s.title,s.artist,s.album,s.album_artist,s.genre,s.year,s.track,s.disc,
                    s.duration,s.cover.as_ref().map(|p|p.to_string_lossy().into_owned()),now(),serde_json::to_string(&s.details)?])?;
        }
        for song in old {
            if song.path.starts_with(root) && !seen.contains(&song.path) {
                tx.execute("DELETE FROM songs WHERE id=?", [song.id])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn favorite(&mut self, id: i64) -> Result<()> {
        if self
            .connection
            .execute("UPDATE songs SET favorite=NOT favorite WHERE id=?", [id])?
            == 0
        {
            bail!("Song no longer exists");
        }
        Ok(())
    }

    pub fn played(&mut self, id: i64) -> Result<()> {
        self.connection.execute(
            "UPDATE songs SET play_count=play_count+1,last_played=? WHERE id=?",
            params![now(), id],
        )?;
        Ok(())
    }

    pub fn save_preferences(&mut self, prefs: &Preferences) -> Result<()> {
        self.connection.execute("INSERT INTO settings(key,value) VALUES('preferences',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [serde_json::to_string(prefs)?])?;
        Ok(())
    }

    pub fn save_session(&mut self, session: &QueueState) -> Result<()> {
        self.connection.execute("INSERT INTO settings(key,value) VALUES('session',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [serde_json::to_string(session)?])?;
        Ok(())
    }

    pub fn reset(&mut self, remove_folders: bool) -> Result<()> {
        let tx = self.connection.transaction()?;
        tx.execute_batch("DELETE FROM playlist_entries; DELETE FROM playlists; DELETE FROM songs; DELETE FROM settings WHERE key='session';")?;
        if remove_folders {
            tx.execute("DELETE FROM roots", [])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_root(&mut self, old: &Path, new: &Path) -> Result<()> {
        if !new.is_dir() {
            bail!("Music folder does not exist: {}", new.display());
        }
        let new = dunce::canonicalize(new)?;
        let snapshot = self.snapshot()?;
        if !snapshot.roots.iter().any(|r| r == old) {
            bail!("Folder no longer exists in your library");
        }
        if old == new {
            return Ok(());
        }
        let tx = self.connection.transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO roots(path) VALUES(?)",
            [new.to_string_lossy().as_ref()],
        )?;
        tx.execute(
            "DELETE FROM roots WHERE path=?",
            [old.to_string_lossy().as_ref()],
        )?;
        for song in snapshot.songs {
            if let Ok(relative) = song.path.strip_prefix(old) {
                let replacement = new.join(relative);
                if replacement.is_file() {
                    if self::destination_conflict(&tx, &replacement, song.id)? {
                        bail!(
                            "The destination already contains tracks in your library. Add it as a folder and remove the old folder instead."
                        );
                    }
                    tx.execute(
                        "UPDATE songs SET path=?,metadata='{}' WHERE id=?",
                        params![replacement.to_string_lossy(), song.id],
                    )?;
                } else if !snapshot
                    .roots
                    .iter()
                    .any(|r| r != old && song.path.starts_with(r))
                {
                    tx.execute("DELETE FROM songs WHERE id=?", [song.id])?;
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn create_playlist(&mut self, name: &str) -> Result<i64> {
        let name = valid_name(name)?;
        self.connection
            .execute("INSERT INTO playlists(name) VALUES(?)", [name])?;
        Ok(self.connection.last_insert_rowid())
    }

    pub fn rename_playlist(&mut self, id: i64, name: &str) -> Result<()> {
        let name = valid_name(name)?;
        if self
            .connection
            .execute("UPDATE playlists SET name=? WHERE id=?", params![name, id])?
            == 0
        {
            bail!("Playlist no longer exists");
        }
        Ok(())
    }

    pub fn delete_playlist(&mut self, id: i64) -> Result<()> {
        self.connection
            .execute("DELETE FROM playlists WHERE id=?", [id])?;
        Ok(())
    }

    pub fn add_to_playlist(&mut self, playlist: i64, song: i64) -> Result<()> {
        self.connection.execute("INSERT INTO playlist_entries(playlist_id,song_id,position)
            VALUES(?1,?2,(SELECT COALESCE(MAX(position)+1,0) FROM playlist_entries WHERE playlist_id=?1))", params![playlist,song])?;
        Ok(())
    }

    pub fn remove_entry(&mut self, entry: i64) -> Result<()> {
        self.connection
            .execute("DELETE FROM playlist_entries WHERE id=?", [entry])?;
        Ok(())
    }

    pub fn move_entry(&mut self, playlist: i64, entry: i64, delta: isize) -> Result<()> {
        let mut entries = self
            .connection
            .prepare("SELECT id FROM playlist_entries WHERE playlist_id=? ORDER BY position,id")?
            .query_map([playlist], |r| r.get::<_, i64>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if let Some(index) = entries.iter().position(|id| *id == entry) {
            let target = index.saturating_add_signed(delta).min(entries.len() - 1);
            let moved = entries.remove(index);
            entries.insert(target, moved);
            let tx = self.connection.transaction()?;
            for (position, id) in entries.iter().enumerate() {
                tx.execute(
                    "UPDATE playlist_entries SET position=? WHERE id=?",
                    params![position as i64, id],
                )?;
            }
            tx.commit()?;
        }
        Ok(())
    }
}

fn valid_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if name.is_empty() {
        bail!("Enter a playlist name");
    }
    if name.chars().count() > 120 {
        bail!("Playlist names must be 120 characters or fewer");
    }
    Ok(name)
}

fn destination_conflict(connection: &Connection, path: &Path, id: i64) -> Result<bool> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM songs WHERE path=? AND id!=?)",
        params![path.to_string_lossy(), id],
        |r| r.get(0),
    )?)
}
