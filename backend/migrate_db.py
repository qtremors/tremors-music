import sqlite3
import os
from database import get_app_dir

DB_PATH = os.path.join(get_app_dir(), "music.db")

def migrate():
    if not os.path.exists(DB_PATH):
        print("No music.db found, nothing to migrate.")
        return

    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    
    try:
        # Check if column exists
        cursor.execute("PRAGMA table_info(song)")
        columns = [info[1] for info in cursor.fetchall()]
        
        if "synced_lyrics" not in columns:
            print("Adding 'synced_lyrics' column to 'song' table...")
            cursor.execute("ALTER TABLE song ADD COLUMN synced_lyrics TEXT")
            conn.commit()
            print("Migration successful.")
        else:
            print("'synced_lyrics' column already exists.")
            
    except Exception as e:
        print(f"Migration error: {e}")
    finally:
        conn.close()

if __name__ == "__main__":
    migrate()
