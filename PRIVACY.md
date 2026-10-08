# Tremors Music - Privacy Policy

## Local processing

Tremors Music processes music locally. It has no accounts, telemetry, uploads, cloud sync, online metadata lookup or automatic update checks. Icons are bundled; no backend server is needed.

The app reads selected music folders, metadata, embedded artwork and matching `.lrc` lyric files. Music and lyric files are never edited or deleted. Automatic folder updates watch only registered roots and can be disabled.

## Stored data

Its local data directory under Windows local app data or Linux XDG local data holds `library.sqlite3`, SQLite WAL/shared-memory files, cached covers and `startup.log`. The local startup log records version, platform and diagnostic errors; it is never uploaded. Music folders shows the location; `TREMORS_DATA_DIR` overrides it for testing. The database stores paths, tags, lyrics, playlists, favorites, counts, preferences, queue IDs and playback position.

## Desktop integration

Desktop media controls expose current-track metadata to Windows SMTC or the local Linux session bus. Artwork references are local file URLs; the app does not fetch remote artwork.

## Your choices

Removing the executable leaves library data in your user profile. Reset controls erase application records without deleting music. To erase all app data, close the app and remove the displayed data directory. Copying it backs up the library, but music paths must remain valid.

## Build and website requests

Dependency downloads happen when building, separately from normal app use. See [LICENSE.md](LICENSE.md).

The project website reads public release information and statistics from GitHub. These requests are separate from the desktop app, which operates offline.
