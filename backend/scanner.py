import os
from sqlmodel import Session, select
from tinytag import TinyTag
from database import engine
from models import Song, Album

AUDIO_EXTENSIONS = {'.mp3', '.flac', '.m4a', '.wav', '.ogg', '.wma', '.aac', '.alac'}

def scan_directory(root_directory: str):
    if not os.path.exists(root_directory): return

    with Session(engine) as session:
        existing_paths = set(session.exec(select(Song.path)).all())
        existing_albums = session.exec(select(Album)).all()
        album_cache = {(a.title.lower(), a.artist.lower()): a for a in existing_albums}
        new_songs_count = 0

        for root, _, files in os.walk(root_directory):
            for file in files:
                ext = os.path.splitext(file)[1].lower()
                if ext in AUDIO_EXTENSIONS:
                    full_path = os.path.join(root, file)
                    if full_path in existing_paths: continue

                    try:
                        tag = TinyTag.get(full_path)
                        title = (tag.title or file).replace('\x00', '').strip()
                        artist = (tag.artist or "Unknown Artist").replace('\x00', '').strip()
                        album_title = (tag.album or "Unknown Album").replace('\x00', '').strip()
                        album_artist = (tag.albumartist or artist).replace('\x00', '').strip()
                        
                        album_key = (album_title.lower(), album_artist.lower())
                        if album_key not in album_cache:
                            new_album = Album(title=album_title, artist=album_artist)
                            session.add(new_album)
                            session.commit()
                            session.refresh(new_album)
                            album_cache[album_key] = new_album
                        
                        file_size = os.path.getsize(full_path)
                        bitrate = int(tag.bitrate) if tag.bitrate else None
                        sample_rate = tag.samplerate
                        
                        song = Song(
                            title=title, artist=artist, path=full_path,
                            duration=tag.duration or 0.0,
                            track_number=int(tag.track) if tag.track and str(tag.track).isdigit() else None,
                            album_id=album_cache[album_key].id,
                            # Tech Specs
                            file_size=file_size,
                            bitrate=bitrate,
                            sample_rate=sample_rate,
                            format=ext.replace('.', '')
                        )
                        session.add(song)
                        new_songs_count += 1
                        
                        if new_songs_count % 50 == 0: session.commit()
                    except: pass

        session.commit()
        # Cleanup Empty Albums
        all_albums = session.exec(select(Album)).all()
        for alb in all_albums:
            if not session.exec(select(Song).where(Song.album_id == alb.id)).first():
                session.delete(alb)
        session.commit()