import os
from datetime import datetime
from sqlmodel import Session, select
from mutagen import File as MutagenFile
from database import engine
from models import Song, Album
from scanner_progress import scanner_progress

AUDIO_EXTENSIONS = {'.mp3', '.flac', '.m4a', '.wav', '.ogg', '.wma', '.aac', '.alac'}

def safe_get(audio, key, default=None):
    """Safely get a tag value, handling both single and multi-value tags."""
    try:
        value = audio.get(key)
        if value:
            # Return first value if it's a list
            return value[0] if isinstance(value, list) else value
        return default
    except:
        return default

def safe_int(value, default=None):
    """Safely convert to int."""
    try:
        if value is None:
            return default
        # Handle "1/12" format
        if isinstance(value, str) and '/' in value:
            value = value.split('/')[0]
        return int(value) if str(value).strip() else default
    except:
        return default

def clean_string(value):
    """Remove null bytes and strip whitespace. Always returns a string, never None."""
    if value is None:
        return ""
    result = str(value).replace('\x00', '').strip()
    return result if result else ""

def scan_directory(root_directory: str):
    if not os.path.exists(root_directory): return

    # Reset and start progress tracking
    scanner_progress.reset()

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
                    
                    # Update current file being processed
                    scanner_progress.update(files=1, current=file)
                    
                    if full_path in existing_paths: continue

                    try:
                        audio = MutagenFile(full_path, easy=True)
                        if audio is None:
                            scanner_progress.update(errors=1)
                            continue
                        
                        # --- BASIC INFORMATION ---
                        title = clean_string(safe_get(audio, 'title', file)) or file
                        artist = clean_string(safe_get(audio, 'artist', 'Unknown Artist')) or 'Unknown Artist'
                        album_title = clean_string(safe_get(audio, 'album', 'Unknown Album')) or 'Unknown Album'
                        album_artist = clean_string(safe_get(audio, 'albumartist', artist)) or artist
                        
                        # --- ALBUM MANAGEMENT ---
                        album_key = (album_title.lower(), album_artist.lower())
                        if album_key not in album_cache:
                            # Extract album-level metadata
                            album_year = safe_int(safe_get(audio, 'date', '').split('-')[0] if safe_get(audio, 'date') else None)
                            album_genre = clean_string(safe_get(audio, 'genre'))
                            
                            new_album = Album(
                                title=album_title,
                                artist=album_artist,
                                year=album_year,
                                genre=album_genre
                            )
                            session.add(new_album)
                            session.commit()
                            session.refresh(new_album)
                            album_cache[album_key] = new_album
                        
                        # --- PEOPLE & CREDITS ---
                        composer = clean_string(safe_get(audio, 'composer'))
                        conductor = clean_string(safe_get(audio, 'conductor'))
                        lyricist = clean_string(safe_get(audio, 'lyricist'))
                        arranger = clean_string(safe_get(audio, 'arranger'))
                        performer = clean_string(safe_get(audio, 'performer'))
                        remixer = clean_string(safe_get(audio, 'remixer'))
                        engineer = clean_string(safe_get(audio, 'engineer'))
                        producer = clean_string(safe_get(audio, 'producer'))
                        
                        # --- ORGANIZATION & CATALOGING ---
                        track_num = safe_int(safe_get(audio, 'tracknumber'))
                        disc_num = safe_int(safe_get(audio, 'discnumber'))
                        genre = clean_string(safe_get(audio, 'genre'))
                        isrc = clean_string(safe_get(audio, 'isrc'))
                        
                        # --- DATES ---
                        date_str = safe_get(audio, 'date')
                        year = safe_int(date_str.split('-')[0] if date_str else None)
                        release_date = clean_string(date_str) if date_str and len(date_str) >= 10 else None
                        original_date = clean_string(safe_get(audio, 'originaldate'))
                        
                        # --- TECHNICAL AUDIO INFO ---
                        file_size = os.path.getsize(full_path)
                        duration = audio.info.length if audio.info else 0.0
                        bitrate = int(audio.info.bitrate / 1000) if audio.info and hasattr(audio.info, 'bitrate') else None
                        sample_rate = int(audio.info.sample_rate) if audio.info and hasattr(audio.info, 'sample_rate') else None
                        channels = int(audio.info.channels) if audio.info and hasattr(audio.info, 'channels') else None
                        bits_per_sample = int(audio.info.bits_per_sample) if audio.info and hasattr(audio.info, 'bits_per_sample') else None
                        codec = str(audio.info.__class__.__name__) if audio.info else None
                        
                        # --- CONTENT & DESCRIPTION ---
                        lyrics = clean_string(safe_get(audio, 'lyrics'))
                        has_lyrics = lyrics is not None and len(lyrics) > 0
                        comment = clean_string(safe_get(audio, 'comment'))
                        description = clean_string(safe_get(audio, 'description'))
                        language = clean_string(safe_get(audio, 'language'))
                        mood = clean_string(safe_get(audio, 'mood'))
                        
                        # --- MUSICAL INFORMATION ---
                        bpm = safe_int(safe_get(audio, 'bpm'))
                        initial_key = clean_string(safe_get(audio, 'initialkey'))
                        
                        # --- REPLAY GAIN ---
                        def safe_float(value, default=None):
                            try:
                                return float(value) if value else default
                            except:
                                return default
                        
                        rg_track_gain = safe_float(safe_get(audio, 'replaygain_track_gain', '').replace(' dB', ''))
                        rg_track_peak = safe_float(safe_get(audio, 'replaygain_track_peak'))
                        rg_album_gain = safe_float(safe_get(audio, 'replaygain_album_gain', '').replace(' dB', ''))
                        rg_album_peak = safe_float(safe_get(audio, 'replaygain_album_peak'))
                        
                        # --- MEDIA TYPE ---
                        media_type = clean_string(safe_get(audio, 'mediatype', 'song'))
                        grouping = clean_string(safe_get(audio, 'grouping'))
                        subtitle = clean_string(safe_get(audio, 'subtitle'))
                        
                        # --- CREATE SONG ---
                        song = Song(
                            # Basic
                            title=title,
                            artist=artist,
                            album_id=album_cache[album_key].id,
                            path=full_path,
                            # People
                            composer=composer,
                            conductor=conductor,
                            lyricist=lyricist,
                            arranger=arranger,
                            performer=performer,
                            remixer=remixer,
                            engineer=engineer,
                            producer=producer,
                            # Organization
                            track_number=track_num,
                            disc_number=disc_num,
                            genre=genre,
                            isrc=isrc,
                            # Dates
                            year=year,
                            release_date=release_date,
                            original_date=original_date,
                            # Technical
                            duration=duration,
                            file_size=file_size,
                            bitrate=bitrate,
                            sample_rate=sample_rate,
                            channels=channels,
                            bits_per_sample=bits_per_sample,
                            format=ext.replace('.', ''),
                            codec=codec,
                            # Content
                            has_lyrics=has_lyrics,
                            lyrics=lyrics,
                            comment=comment,
                            description=description,
                            language=language,
                            mood=mood,
                            # Musical
                            bpm=bpm,
                            initial_key=initial_key,
                            # ReplayGain
                            replaygain_track_gain=rg_track_gain,
                            replaygain_track_peak=rg_track_peak,
                            replaygain_album_gain=rg_album_gain,
                            replaygain_album_peak=rg_album_peak,
                            # Media
                            media_type=media_type,
                            grouping=grouping,
                            subtitle=subtitle,
                            # User data
                            date_added=datetime.now().isoformat()
                        )
                        session.add(song)
                        new_songs_count += 1
                        scanner_progress.update(songs=1)  # Track song added
                        
                        if new_songs_count % 50 == 0:
                            session.commit()
                            
                    except Exception as e:
                        # Track detailed error information
                        error_msg = str(e) if str(e) else "Unknown error"
                        scanner_progress.update(
                            errors=1,
                            error_file=full_path,
                            error_msg=error_msg
                        )
                        pass

        session.commit()
        scanner_progress.finish()  # Mark scanning as complete
        
        # Cleanup Empty Albums
        all_albums = session.exec(select(Album)).all()
        for alb in all_albums:
            if not session.exec(select(Song).where(Song.album_id == alb.id)).first():
                session.delete(alb)
        session.commit()
        print(f"Scan complete: {new_songs_count} new songs added")