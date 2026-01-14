# Privacy Notice - Tremors Music

> **Project:** Tremors Music  
> **Version:** 2.0.1  
> **Last Updated:** 2026-01-13

---

Tremors Music is designed with privacy as a core principle. Here's how we protect your data:

> [!IMPORTANT]
> **Complete Privacy Guarantee**  
> Tremors Music is a **100% local-first application**. Your music files, playlists, and settings never leave your machine. No accounts, no cloud, no tracking.

---

## 🔒 Local-Only Operation

All data generated or processed by the application is stored locally on your device.

### What We DON'T Do

- ❌ **No Data Collection**: We do not collect any personal information (names, emails, etc.).
- ❌ **No Tracking**: We do not track your listening habits or search queries.
- ❌ **No Telemetry**: We do not send analytics, crash reports, or telemetry to any server.
- ❌ **No Accounts**: There are no user accounts, logins, or cloud synchronization.
- ❌ **No Uploads**: Your music files are never uploaded to any remote server or service.

### What The App DOES Store (Locally)

All data is stored in the application's installation/data directory:

| File/Folder | Purpose |
|-------------|---------|
| `music.db` | SQLite database containing your music library metadata |
| `logs/` | Local application logs for debugging and troubleshooting |
| `covers/` | Cached album artwork extracted from your local music files |

---

## 🌐 Network Requests

The application makes **zero network requests** during normal operation. 

- **Lyrics**: Extracted directly from embedded ID3/M4A tags in your files.
- **Updates**: No automatic update checking is performed.
- **Asset Loading**: All icons and UI elements are bundled within the app.

---

## 📝 Your Rights

Since all data remains in your control on your local machine:
- **Access**: You can open the `music.db` file with any SQLite browser to see exactly what is stored.
- **Portability**: You can copy the application directory to move your library and settings to another computer.
- **Erasure**: Deleting the application directory or uninstalling the app removes all associated data permanently.

---

<p align="center">
  <a href="README.md">← Back to README</a>
</p>

<p align="center">
  Made with ❤️ by <a href="https://github.com/qtremors">Tremors</a>
</p>
