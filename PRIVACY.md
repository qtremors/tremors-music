# Tremors Music - Privacy Policy

## Local processing

Tremors Music processes music locally. It has no accounts, telemetry, uploads, cloud sync, online metadata lookup or automatic update checks. Icons are bundled; no backend server is needed.

The app reads selected music folders, metadata, embedded artwork and matching `.lrc`, `.ttml`, and `.txt` lyric files. Browsing and scanning never modify music. The metadata editor writes tags, embedded lyrics, and images only when you choose **Save to file**. It checks for outside file changes and validates a temporary copy before replacing the original. The local lyric editor writes a matching sidecar only when you choose **Save lyrics**, with an outside-change check. Folder removal and library resets never delete music. Automatic folder updates watch only registered roots and can be disabled.

## Stored data

Its local data directory under Windows local app data or Linux XDG local data holds `library.sqlite3`, SQLite WAL/shared-memory files, cached covers, chosen local artist/playlist images and `startup.log`. The local startup log records version, platform and diagnostic errors; it is never uploaded. Music folders shows the location; `TREMORS_DATA_DIR` overrides it for testing. The database stores paths, tags, lyrics, playlists, favorites, counts, preferences, queue IDs and playback position.

## Desktop integration

Desktop media controls expose current-track metadata to Windows SMTC or the local Linux session bus. Artwork references are local file URLs; the app does not fetch remote artwork.

## Your choices

Removing the executable leaves library data in your user profile. Reset controls erase application records without deleting music. To erase all app data, close the app and remove the displayed data directory. Copying it backs up the library, but music paths must remain valid. **Settings → Backup & restore** also saves a local curation archive containing playlists, favorites, play history, preferences, music paths, and chosen artwork. It excludes the music files and is never uploaded. Restore can map a moved folder, adds playlists alongside existing ones, and merges curation.

## Build and website requests

Dependency downloads happen when building, separately from normal app use. See [LICENSE.md](LICENSE.md).

The project website reads public release information and statistics from GitHub. These requests are separate from the desktop app, which operates offline.
