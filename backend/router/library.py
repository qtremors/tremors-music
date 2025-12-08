from typing import List, Dict, Any
from fastapi import APIRouter, Depends, BackgroundTasks, HTTPException
from sqlmodel import Session, select, func, or_, delete
from database import get_session
from models import Song, Album, LibraryPath
from scanner import scan_directory
import os
import urllib.parse

router = APIRouter(prefix="/library", tags=["Library"])

# --- PATH MANAGEMENT ---
@router.get("/paths", response_model=List[LibraryPath])
def get_paths(session: Session = Depends(get_session)):
    return session.exec(select(LibraryPath)).all()

@router.post("/paths")
def add_path(path_data: LibraryPath, session: Session = Depends(get_session)):
    if not os.path.exists(path_data.path):
        raise HTTPException(status_code=400, detail="Directory does not exist")
    existing = session.exec(select(LibraryPath).where(LibraryPath.path == path_data.path)).first()
    if existing: return existing
    session.add(path_data)
    session.commit()
    session.refresh(path_data)
    return path_data

@router.delete("/paths/{path_id}")
def remove_path(path_id: int, session: Session = Depends(get_session)):
    path_obj = session.get(LibraryPath, path_id)
    if not path_obj: raise HTTPException(status_code=404, detail="Path not found")
    session.delete(path_obj)
    session.commit()
    return {"message": "Path removed"}

@router.patch("/paths/{path_id}")
def update_library_path(path_id: int, new_path: str, session: Session = Depends(get_session)):
    """Update an existing library path."""
    path_obj = session.get(LibraryPath, path_id)
    if not path_obj:
        raise HTTPException(status_code=404, detail="Path not found")
    if not os.path.exists(new_path):
        raise HTTPException(status_code=400, detail="Directory does not exist")
    path_obj.path = new_path
    session.commit()
    session.refresh(path_obj)
    return path_obj

@router.patch("/paths/{path_id}")
def update_library_path(path_id: int, new_path: str, session: Session = Depends(get_session)):
    """Update an existing library path."""
    path_obj = session.get(LibraryPath, path_id)
    if not path_obj:
        raise HTTPException(status_code=404, detail="Path not found")
    if not os.path.exists(new_path):
        raise HTTPException(status_code=400, detail="Directory does not exist")
    existing = session.exec(select(LibraryPath).where(LibraryPath.path == new_path)).first()
    if existing and existing.id != path_id:
        raise HTTPException(status_code=400, detail="Path already exists in library")
    path_obj.path = new_path
    session.commit()
    session.refresh(path_obj)
    return path_obj

# --- LIBRARY MANAGEMENT ---
@router.delete("/reset")
def reset_library(session: Session = Depends(get_session)):
    """Wipes Songs and Albums but KEEPS Paths."""
    session.exec(delete(Song))
    session.exec(delete(Album))
    session.commit()
    return {"message": "Library reset (Paths saved)"}

@router.post("/scan")
def scan_library(background_tasks: BackgroundTasks, session: Session = Depends(get_session)):
    paths = session.exec(select(LibraryPath)).all()
    if not paths: raise HTTPException(status_code=400, detail="No paths configured.")
    
    # Import here to avoid circular dependency
    from scanner_progress import scanner_progress
    
    # Reset progress before starting
    scanner_progress.reset()
    
    for p in paths:
        background_tasks.add_task(scan_directory, os.path.normpath(p.path))
    return {"message": "Scanning started"}

@router.get("/scan/status")
def get_scan_status():
    """Get real-time scanner progress"""
    from scanner_progress import scanner_progress
    return scanner_progress.to_dict()

@router.post("/scan/stop")
def stop_scan():
    """Stop the currently running scan."""
    from scanner_progress import scanner_progress
    if not scanner_progress.is_scanning:
        raise HTTPException(status_code=400, detail="No scan is currently running")
    scanner_progress.finish()
    return {"message": "Scan stopped"}

@router.post("/scan/stop")
def stop_scan():
    """Stop the currently running scan."""
    from scanner_progress import scanner_progress
    if not scanner_progress.is_scanning:
        raise HTTPException(status_code=400, detail="No scan is currently running")
    scanner_progress.finish()
    return {"message": "Scan stopped"}

# --- ARTISTS ---
@router.get("/artists")
def get_artists(session: Session = Depends(get_session)):
    albums = session.exec(select(Album)).all()
    artist_map = {}
    for album in albums:
        if album.artist not in artist_map:
            artist_map[album.artist] = {
                "name": album.artist,
                "album_count": 0,
                "cover_example": album.id 
            }
        artist_map[album.artist]["album_count"] += 1
    return list(artist_map.values())

@router.get("/artists/{artist_name}/work")
def get_artist_work(artist_name: str, session: Session = Depends(get_session)):
    decoded_name = urllib.parse.unquote(artist_name).lower()
    
    # Join with Album to check Album Artist too
    songs = session.exec(
        select(Song)
        .join(Album, isouter=True)
        .where(
            or_(
                func.lower(Song.artist) == decoded_name,
                func.lower(Album.artist) == decoded_name
            )
        )
        .order_by(Song.album_id, Song.track_number)
    ).all()
    
    work_map = {}
    
    for song in songs:
        alb_id = song.album_id or -1
        if alb_id not in work_map:
            album_data = None
            if alb_id != -1:
                album_data = session.get(Album, alb_id)
            
            work_map[alb_id] = {
                "album": album_data,
                "songs": []
            }
        work_map[alb_id]["songs"].append(song)
        
    return list(work_map.values())

# --- GENRES ---
@router.get("/genres")
def get_genres(session: Session = Depends(get_session)):
    """Get all unique genres with song counts."""
    # Get all non-null genres from songs
    songs = session.exec(select(Song).where(Song.genre.is_not(None))).all()
    
    genre_map = {}
    for song in songs:
        if song.genre:
            # Split multi-genre tags (common separators: comma, semicolon)
            # Don't split on / to keep genres like "R&B/Soul" together
            import re
            genres = re.split(r'[,;]', song.genre)
            for genre in genres:
                genre = genre.strip()
                if genre:
                    if genre not in genre_map:
                        genre_map[genre] = {"name": genre, "song_count": 0}
                    genre_map[genre]["song_count"] += 1
    
    # Sort by song count descending
    return sorted(genre_map.values(), key=lambda x: x["song_count"], reverse=True)

@router.get("/genres/songs", response_model=List[Song])
def get_genre_songs(name: str, session: Session = Depends(get_session)):
    """Get all songs for a specific genre using query parameter."""
    import re
    
    # Get all songs with genre tags
    all_songs = session.exec(select(Song).where(Song.genre.is_not(None))).all()
    
    # Filter songs that have this exact genre (handling multi-genre tags)
    matching_songs = []
    for song in all_songs:
        if song.genre:
            # Split by comma/semicolon and check for exact match
            song_genres = [g.strip() for g in re.split(r'[,;]', song.genre)]
            if name in song_genres:
                matching_songs.append(song)
    
    # Sort by artist then title
    matching_songs.sort(key=lambda s: (s.artist or '', s.title or ''))
    return matching_songs


# --- SMART PLAYLISTS ---
@router.get("/smart-playlists/favorites", response_model=List[Song])
def get_favorites(limit: int = 100, session: Session = Depends(get_session)):
    """Get favorite songs (rating >= 4) or all songs if no ratings set."""
    # First try to get songs with high ratings
    songs = session.exec(
        select(Song)
        .where(Song.rating >= 4)
        .order_by(Song.rating.desc(), Song.play_count.desc())
        .limit(limit)
    ).all()
    
    # If no rated songs, return most played instead
    if len(songs) == 0:
        songs = session.exec(
            select(Song)
            .where(Song.play_count > 0)
            .order_by(Song.play_count.desc())
            .limit(limit)
        ).all()
    
    return songs


@router.get("/smart-playlists/recently-added", response_model=List[Song])
def get_recently_added(limit: int = 50, session: Session = Depends(get_session)):
    """Get recently added songs, sorted by id desc (newest first)."""
    songs = session.exec(
        select(Song)
        .order_by(Song.id.desc())
        .limit(limit)
    ).all()
    return songs


@router.get("/smart-playlists/most-played", response_model=List[Song])
def get_most_played(limit: int = 50, session: Session = Depends(get_session)):
    """Get most played songs."""
    songs = session.exec(
        select(Song)
        .where(Song.play_count > 0)
        .order_by(Song.play_count.desc())
        .limit(limit)
    ).all()
    return songs


@router.post("/songs/{song_id}/play")
def increment_play_count(song_id: int, session: Session = Depends(get_session)):
    """Increment play count for a song."""
    song = session.get(Song, song_id)
    if not song:
        raise HTTPException(status_code=404, detail="Song not found")
    song.play_count = (song.play_count or 0) + 1
    session.commit()
    return {"play_count": song.play_count}


@router.post("/songs/{song_id}/favorite")
def toggle_favorite(song_id: int, session: Session = Depends(get_session)):
    """Toggle favorite status (rating 5 = favorite, 0 = not favorite)."""
    song = session.get(Song, song_id)
    if not song:
        raise HTTPException(status_code=404, detail="Song not found")
    # Toggle: if rated 5, set to 0; otherwise set to 5
    song.rating = 0 if song.rating == 5 else 5
    session.commit()
    return {"rating": song.rating, "is_favorite": song.rating == 5}


# --- SONGS & SORTING ---
@router.get("/songs", response_model=List[Song])
def get_songs(
    offset: int = 0, 
    limit: int = 5000, 
    sort_by: str = "title", 
    order: str = "asc", 
    session: Session = Depends(get_session)
):
    query = select(Song)
    
    # Case-insensitive sorting logic
    if sort_by == "title":
        sort_col = func.lower(Song.title)
    elif sort_by == "artist":
        sort_col = func.lower(Song.artist)
    elif sort_by == "album":
        query = query.join(Album, isouter=True)
        sort_col = func.lower(Album.title)
    elif sort_by == "duration":
        sort_col = Song.duration
    elif sort_by == "year":
        sort_col = Song.year
    elif sort_by == "file_size":
        sort_col = Song.file_size
    elif sort_by == "id":
        sort_col = Song.id
    else:
        sort_col = func.lower(Song.title)

    if order == "desc":
        query = query.order_by(sort_col.desc())
    else:
        query = query.order_by(sort_col.asc())

    return session.exec(query.offset(offset).limit(limit)).all()

# --- SEARCH ---
@router.get("/search", response_model=Dict[str, Any])
def search_library(q: str = "", limit: int = 50, session: Session = Depends(get_session)):
    """
    Smart search across songs, albums, and artists with relevance scoring.
    Works like a real music player search - prioritizes direct title matches.
    
    Scoring logic (higher is better):
    - Exact title/name match: 1000 points
    - Title/name starts with query: 500 points  
    - Title/name contains query: 200 points
    - Artist field matches: 50 points (secondary)
    
    Key principle: Songs are only matched by their OWN title/artist, not by album.
    """
    if not q:
        return {"songs": [], "albums": [], "artists": [], "bestMatchType": None}
    
    q_lower = q.lower().strip()
    search_term = f"%{q}%"
    
    def score_text(value: str, multiplier: int = 1) -> int:
        """Calculate relevance score for text matching."""
        if not value:
            return 0
        val_lower = value.lower()
        if val_lower == q_lower:
            return 1000 * multiplier  # Exact match
        elif val_lower.startswith(q_lower):
            return 500 * multiplier  # Starts with
        elif q_lower in val_lower:
            return 200 * multiplier  # Contains
        return 0
    
    # === SONGS ===
    # Only match songs by their OWN title or artist - NOT by album title
    songs_query = session.exec(
        select(Song).where(
            or_(
                Song.title.ilike(search_term),
                Song.artist.ilike(search_term)
            )
        ).limit(limit)
    ).all()
    
    # Score songs: title match is primary, artist is secondary
    scored_songs = []
    for song in songs_query:
        title_score = score_text(song.title, multiplier=2)  # Title weighted 2x
        artist_score = score_text(song.artist, multiplier=1)
        
        # Use title score as primary, add small artist bonus
        if title_score > 0:
            total_score = title_score + (artist_score // 10)  # Small bonus for artist match
        else:
            total_score = artist_score  # Only artist matched
        
        scored_songs.append((song, total_score, title_score > 0))  # Track if title matched
    
    # Sort: first by whether title matched, then by score
    scored_songs.sort(key=lambda x: (x[2], x[1]), reverse=True)
    songs = [s[0] for s in scored_songs]
    best_song_score = scored_songs[0][1] if scored_songs else 0
    best_song_title_match = scored_songs[0][2] if scored_songs else False
    
    # === ALBUMS ===
    albums_query = session.exec(
        select(Album).where(
            or_(
                Album.title.ilike(search_term),
                Album.artist.ilike(search_term)
            )
        ).limit(20)
    ).all()
    
    scored_albums = []
    for album in albums_query:
        title_score = score_text(album.title, multiplier=2)
        artist_score = score_text(album.artist, multiplier=1)
        
        if title_score > 0:
            total_score = title_score + (artist_score // 10)
        else:
            total_score = artist_score
        
        scored_albums.append((album, total_score, title_score > 0))
    
    scored_albums.sort(key=lambda x: (x[2], x[1]), reverse=True)
    albums = [a[0] for a in scored_albums]
    best_album_score = scored_albums[0][1] if scored_albums else 0
    best_album_title_match = scored_albums[0][2] if scored_albums else False
    
    # === ARTISTS ===
    # Split combined artist names into individual artists
    def split_artists(artist_str: str) -> list:
        """Split combined artist names by common separators."""
        if not artist_str:
            return []
        # Common separators: comma, &, feat., ft., featuring, and, with
        import re
        # Split by common separators
        parts = re.split(r'\s*[,&]\s*|\s+feat\.?\s+|\s+ft\.?\s+|\s+featuring\s+|\s+and\s+|\s+with\s+', artist_str, flags=re.IGNORECASE)
        return [p.strip() for p in parts if p.strip()]
    
    song_artists = session.exec(
        select(Song.artist).where(
            Song.artist.ilike(search_term)
        ).distinct()
    ).all()
    
    album_artists = session.exec(
        select(Album.artist).where(
            Album.artist.ilike(search_term)
        ).distinct()
    ).all()
    
    # Collect and split all artist names
    all_artist_names = set()
    for artist in song_artists:
        if artist:
            # Split combined artists into individuals
            for individual in split_artists(artist):
                if q_lower in individual.lower():
                    all_artist_names.add(individual)
    for artist in album_artists:
        if artist:
            for individual in split_artists(artist):
                if q_lower in individual.lower():
                    all_artist_names.add(individual)
    
    scored_artists = []
    for artist_name in all_artist_names:
        artist_score = score_text(artist_name, multiplier=2)
        scored_artists.append(({"name": artist_name}, artist_score))
    
    scored_artists.sort(key=lambda x: x[1], reverse=True)
    artists = [a[0] for a in scored_artists[:20]]
    best_artist_score = scored_artists[0][1] if scored_artists else 0
    
    # === DETERMINE BEST MATCH TYPE ===
    # Priority: Direct title/name matches beat secondary matches
    best_match_type = None
    
    # Check for direct title matches first (highest priority)
    if best_song_title_match and best_song_score >= best_album_score and best_song_score >= best_artist_score:
        best_match_type = "song"
    elif best_album_title_match and best_album_score >= best_song_score and best_album_score >= best_artist_score:
        best_match_type = "album"
    elif best_artist_score >= 500:  # Artist name starts with or exact match
        best_match_type = "artist"
    # Fallback: highest absolute score
    elif best_song_score >= best_album_score and best_song_score >= best_artist_score and best_song_score > 0:
        best_match_type = "song"
    elif best_album_score >= best_artist_score and best_album_score > 0:
        best_match_type = "album"
    elif best_artist_score > 0:
        best_match_type = "artist"
    
    return {
        "songs": songs,
        "albums": albums,
        "artists": artists,
        "bestMatchType": best_match_type,
        "scores": {
            "song": best_song_score,
            "album": best_album_score,
            "artist": best_artist_score
        }
    }

# --- ALBUMS ---
@router.get("/albums", response_model=List[Album])
def get_albums(offset: int = 0, limit: int = 50, session: Session = Depends(get_session)):
    return session.exec(select(Album).offset(offset).limit(limit)).all()

@router.get("/albums/{album_id}")
def get_album_details(album_id: int, session: Session = Depends(get_session)):
    return session.get(Album, album_id)

@router.get("/albums/{album_id}/songs", response_model=List[Song])
def get_album_songs(album_id: int, session: Session = Depends(get_session)):
    return session.exec(select(Song).where(Song.album_id == album_id).order_by(Song.track_number, Song.title)).all()