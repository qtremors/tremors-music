# Privacy Notice

**Last Updated: December 2025**

Tremors Music is designed with privacy as a core principle. Here's what you need to know:

## 🔒 Local-Only Operation

Tremors Music is a **100% local application**. Your music files, playlists, and settings never leave your computer.

### What We DON'T Do

- ❌ We do NOT collect any personal data
- ❌ We do NOT track your listening habits
- ❌ We do NOT send analytics or telemetry
- ❌ We do NOT require an internet connection to function
- ❌ We do NOT have user accounts or authentication
- ❌ We do NOT upload your music files anywhere

### What The App DOES Store (Locally)

All data is stored in the application's installation directory:

| File/Folder | Purpose |
|-------------|---------|
| `music.db` | SQLite database containing your music library metadata |
| `logs/` | Application logs for debugging (stored locally only) |
| `covers/` | Cached album artwork extracted from your files |

### Network Requests

The application makes **NO network requests** during normal operation. All features work completely offline.

- Lyrics are extracted from embedded tags during library scanning
- No analytics, telemetry, or tracking
- No data is ever uploaded

### Data Portability

Since all data is stored in the installation folder, you can:
- Move the entire folder to another computer
- Back up your library by copying the folder
- Delete all data by simply uninstalling the application

### Your Rights

You have complete control over your data:
- **Access**: All data is in readable formats (SQLite, logs)
- **Delete**: Uninstall removes all application data
- **Portability**: Copy the folder to move between computers

---

**Questions?** Open an issue on [GitHub](https://github.com/qtremors/tremors-music)
