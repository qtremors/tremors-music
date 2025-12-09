import os
from datetime import datetime
from sqlmodel import Session, select
from mutagen import File as MutagenFile
from mutagen.id3 import ID3
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
    except (TypeError, KeyError, AttributeError):
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
    except (ValueError, TypeError, AttributeError):
        return default

def safe_float(value, default=None):
    """Safely convert to float."""
    try:
        return float(value) if value else default
    except (ValueError, TypeError):
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

    try:
        with Session(engine) as session:
            # Load existing set for fast lookup
            existing_songs = session.exec(select(Song)).all()
            # Map path -> Song object
            song_map = {s.path: s for s in existing_songs}
            
            found_paths = set()
            existing_albums = session.exec(select(Album)).all()
            album_cache = {(a.title.lower(), a.artist.lower()): a for a in existing_albums}
            
            new_songs_count = 0
            updated_songs_count = 0
            deleted_songs_count = 0

            for root, _, files in os.walk(root_directory):
                # Check cancellation
                if not scanner_progress.is_scanning:
                    break

                for file in files:
                    # Check cancellation
                    if not scanner_progress.is_scanning:
                        break

                    ext = os.path.splitext(file)[1].lower()
                    if ext in AUDIO_EXTENSIONS:
                        full_path = os.path.join(root, file)
                        found_paths.add(full_path)
                        
                        # Update current file being processed
                        scanner_progress.update(files=1, current=file)
                        
                        try:
                            file_size = os.path.getsize(full_path)
                            
                            # Check if update is needed
                            existing_song = song_map.get(full_path)
                            if existing_song:
                                # If file size hasn't changed, skip parsing (optimization)
                                # BUT -> If missing lyrics, force re-scan to apply new robust extraction logic
                                if existing_song.file_size == file_size and existing_song.has_lyrics:
                                    continue
                            
                            # Parse Tags (for New OR Update)
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
                            
                            album_id = album_cache[album_key].id

                            # --- METADATA EXTRACTION ---
                            # People
                            composer = clean_string(safe_get(audio, 'composer'))
                            conductor = clean_string(safe_get(audio, 'conductor'))
                            lyricist = clean_string(safe_get(audio, 'lyricist'))
                            arranger = clean_string(safe_get(audio, 'arranger'))
                            performer = clean_string(safe_get(audio, 'performer'))
                            remixer = clean_string(safe_get(audio, 'remixer'))
                            engineer = clean_string(safe_get(audio, 'engineer'))
                            producer = clean_string(safe_get(audio, 'producer'))
                            
                            # Organization
                            track_num = safe_int(safe_get(audio, 'tracknumber'))
                            disc_num = safe_int(safe_get(audio, 'discnumber'))
                            genre = clean_string(safe_get(audio, 'genre'))
                            isrc = clean_string(safe_get(audio, 'isrc'))
                            
                            # Dates
                            date_str = safe_get(audio, 'date')
                            year = safe_int(date_str.split('-')[0] if date_str else None)
                            release_date = clean_string(date_str) if date_str and len(date_str) >= 10 else None
                            original_date = clean_string(safe_get(audio, 'originaldate'))
                            
                            # Technical
                            duration = audio.info.length if audio.info else 0.0
                            bitrate = int(audio.info.bitrate / 1000) if audio.info and hasattr(audio.info, 'bitrate') else None
                            sample_rate = int(audio.info.sample_rate) if audio.info and hasattr(audio.info, 'sample_rate') else None
                            channels = int(audio.info.channels) if audio.info and hasattr(audio.info, 'channels') else None
                            bits_per_sample = int(audio.info.bits_per_sample) if audio.info and hasattr(audio.info, 'bits_per_sample') else None
                            codec = str(audio.info.__class__.__name__) if audio.info else None
                            
                            # Content
                            lyrics = clean_string(safe_get(audio, 'lyrics'))
                            
                            # Fallback for ID3 USLT if easy=True missed it
                            if not lyrics and (ext == '.mp3' or ext == '.m4a'):
                                try:
                                    audio_raw = MutagenFile(full_path)
                                    if audio_raw and hasattr(audio_raw, 'tags'):
                                        # ID3 USLT
                                        if isinstance(audio_raw.tags, ID3):
                                            for key in audio_raw.tags.keys():
                                                if key.startswith('USLT'):
                                                    frame = audio_raw.tags[key]
                                                    lyrics = getattr(frame, 'text', str(frame))
                                                    break
                                        # M4A ©lyr
                                        elif '©lyr' in audio_raw:
                                            lyrics = audio_raw['©lyr'][0]
                                except:
                                    pass

                            has_lyrics = lyrics is not None and len(lyrics) > 0
                            comment = clean_string(safe_get(audio, 'comment'))
                            description = clean_string(safe_get(audio, 'description'))
                            language = clean_string(safe_get(audio, 'language'))
                            mood = clean_string(safe_get(audio, 'mood'))
                            
                            # Musical
                            bpm = safe_int(safe_get(audio, 'bpm'))
                            initial_key = clean_string(safe_get(audio, 'initialkey'))
                            
                            rg_track_gain = safe_float(safe_get(audio, 'replaygain_track_gain', '').replace(' dB', ''))
                            rg_track_peak = safe_float(safe_get(audio, 'replaygain_track_peak'))
                            rg_album_gain = safe_float(safe_get(audio, 'replaygain_album_gain', '').replace(' dB', ''))
                            rg_album_peak = safe_float(safe_get(audio, 'replaygain_album_peak'))
                            
                            # Media
                            media_type = clean_string(safe_get(audio, 'mediatype', 'song'))
                            grouping = clean_string(safe_get(audio, 'grouping'))
                            subtitle = clean_string(safe_get(audio, 'subtitle'))

                            if existing_song:
                                # --- UPDATE EXISTING ---
                                existing_song.title = title
                                existing_song.artist = artist
                                existing_song.album_id = album_id
                                existing_song.file_size = file_size
                                
                                # Bulk update fields
                                existing_song.composer = composer
                                existing_song.conductor = conductor
                                existing_song.lyricist = lyricist
                                existing_song.arranger = arranger
                                existing_song.performer = performer
                                existing_song.remixer = remixer
                                existing_song.engineer = engineer
                                existing_song.producer = producer
                                existing_song.track_number = track_num
                                existing_song.disc_number = disc_num
                                existing_song.genre = genre
                                existing_song.isrc = isrc
                                existing_song.year = year
                                existing_song.release_date = release_date
                                existing_song.original_date = original_date
                                existing_song.duration = duration
                                existing_song.bitrate = bitrate
                                existing_song.sample_rate = sample_rate
                                existing_song.channels = channels
                                existing_song.bits_per_sample = bits_per_sample
                                existing_song.codec = codec
                                existing_song.has_lyrics = has_lyrics
                                existing_song.lyrics = lyrics
                                # Simple detection: if lyrics contains [00:00 style timestamps, it might be synced
                                if lyrics and '[' in lyrics and ']' in lyrics:
                                    existing_song.synced_lyrics = lyrics
                                existing_song.comment = comment
                                existing_song.description = description
                                existing_song.language = language
                                existing_song.mood = mood
                                existing_song.bpm = bpm
                                existing_song.initial_key = initial_key
                                existing_song.replaygain_track_gain = rg_track_gain
                                existing_song.replaygain_track_peak = rg_track_peak
                                existing_song.replaygain_album_gain = rg_album_gain
                                existing_song.replaygain_album_peak = rg_album_peak
                                existing_song.media_type = media_type
                                existing_song.grouping = grouping
                                existing_song.subtitle = subtitle
                                
                                session.add(existing_song)
                                updated_songs_count += 1
                            else:
                                # --- INSERT NEW ---
                                song = Song(
                                    title=title,
                                    artist=artist,
                                    album_id=album_id,
                                    path=full_path,
                                    composer=composer,
                                    conductor=conductor,
                                    lyricist=lyricist,
                                    arranger=arranger,
                                    performer=performer,
                                    remixer=remixer,
                                    engineer=engineer,
                                    producer=producer,
                                    track_number=track_num,
                                    disc_number=disc_num,
                                    genre=genre,
                                    isrc=isrc,
                                    year=year,
                                    release_date=release_date,
                                    original_date=original_date,
                                    duration=duration,
                                    file_size=file_size,
                                    bitrate=bitrate,
                                    sample_rate=sample_rate,
                                    channels=channels,
                                    bits_per_sample=bits_per_sample,
                                    format=ext.replace('.', ''),
                                    codec=codec,
                                    has_lyrics=has_lyrics,
                                    lyrics=lyrics,
                                    synced_lyrics=lyrics if (lyrics and '[' in lyrics and ']' in lyrics) else None,
                                    comment=comment,
                                    description=description,
                                    language=language,
                                    mood=mood,
                                    bpm=bpm,
                                    initial_key=initial_key,
                                    replaygain_track_gain=rg_track_gain,
                                    replaygain_track_peak=rg_track_peak,
                                    replaygain_album_gain=rg_album_gain,
                                    replaygain_album_peak=rg_album_peak,
                                    media_type=media_type,
                                    grouping=grouping,
                                    subtitle=subtitle,
                                    date_added=datetime.now().isoformat()
                                )
                                session.add(song)
                                new_songs_count += 1
                                scanner_progress.update(songs=1)

                            if (new_songs_count + updated_songs_count) % 50 == 0:
                                session.commit()
                                
                        except Exception as e:
                            # Track detailed error information
                            error_msg = str(e) if str(e) else "Unknown error"
                            scanner_progress.update(
                                errors=1,
                                error_file=full_path,
                                error_msg=error_msg
                            )
                            continue

            # --- CLEANUP DELETED FILES ---
            start_paths = [p for p in song_map.keys() if p.startswith(str(root_directory))]
            for path in start_paths:
                if path not in found_paths:
                    song_to_delete = song_map[path]
                    session.delete(song_to_delete)
                    deleted_songs_count += 1
            
            session.commit()
            
            # Cleanup Empty Albums
            all_albums = session.exec(select(Album)).all()
            for alb in all_albums:
                if not session.exec(select(Song).where(Song.album_id == alb.id)).first():
                    session.delete(alb)
            session.commit()
            
            print(f"Scan complete: {new_songs_count} new, {updated_songs_count} updated, {deleted_songs_count} deleted.")
            
    except Exception as e:
        print(f"Critical Scanner Error: {e}")
        scanner_progress.update(errors=1, error_msg=f"Critical: {str(e)}")
    finally:
        scanner_progress.finish()