from typing import List, Dict, Any
from fastapi import APIRouter, Depends, BackgroundTasks, HTTPException
from sqlmodel import Session, select, func, or_, delete, col
from sqlalchemy import case
from database import get_session
from models import Song, Album, LibraryPath, SongListItem, AlbumRead
from scanner import scan_directory
import os
import re
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
    existing = session.exec(select(LibraryPath).where(LibraryPath.path == new_path)).first()
    if existing and existing.id != path_id:
        raise HTTPException(status_code=400, detail="Path already exists in library")
    path_obj.path = new_path
    session.commit()
    session.refresh(path_obj)
    return path_obj

# --- LIBRARY MANAGEMENT ---
@router.delete("/reset")
def reset_library(
    background_tasks: BackgroundTasks,
    hard: bool = False,
    session: Session = Depends(get_session)
):
    """
    Reset or Rescan the library.
    - Default (hard=False): Performs a "Soft Reset" (Rescan & Prune). 
      This updates existing files, adds new ones, and removes missing ones, 
      while PRESERVING play counts, ratings, and date_added for existing files.
    - hard=True: Wipes all Songs and Albums from the database. Destructive.
    """
    if hard:
        session.exec(delete(Song))
        session.exec(delete(Album))
        session.commit()
        return {"message": "Library WIPED (Hard Reset). Paths saved."}
    else:
        # Perform Smart Rescan (Sync)
        paths = session.exec(select(LibraryPath)).all()
        if not paths:
            return {"message": "No paths to scan."}
        
        # Reset progress tracker
        from scanner_progress import scanner_progress
        scanner_progress.reset()
        
        for p in paths:
            # Re-use the smart scan_directory logic which now handles pruning
            background_tasks.add_task(scan_directory, os.path.normpath(p.path))
            
        return {"message": "Smart Rescan started (Sync & Prune). Use ?hard=true to completely wipe."}

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
    """Get favorite songs (rating == 5 only)."""
    songs = session.exec(
        select(Song)
        .where(Song.rating == 5)
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
    if song.rating == 5:
        song.rating = 0
    else:
        song.rating = 5
    session.add(song)
    session.commit()
    return {"rating": song.rating}


# --- SONGS ---
@router.get("/songs", response_model=List[SongListItem])
def get_songs(
    offset: int = 0, 
    limit: int = 5000, 
    sort_by: str = "title",
    order: str = "asc",
    session: Session = Depends(get_session)
):
    # Use SongListItem to select specific columns (optimization)
    # SQLModel doesn't support 'select(SongListItem)' directly on a mapped table easily without some tricks
    # So we select Song and let Pydantic filter it, OR we explicitly select columns.
    # explicit selection is faster for massive DBs but Pydantic filtering saves memory transfer.
    # Given the goal is offline/speed, let's let fastapi response_model handle filtering for now, 
    # BUT we should select(Song) and just ensure we don't load deferred cols if we had them. 
    # For now, just changing the response_model avoids sending the lyrics over the wire.
    
    query = select(Song)
    
    if sort_by == "artist":
        sort_col = func.lower(Song.artist)
    elif sort_by == "album":
        sort_col = func.lower(Song.album_id) 
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

@router.get("/songs/{song_id}", response_model=Song)
def get_song(song_id: int, session: Session = Depends(get_session)):
    song = session.get(Song, song_id)
    if not song: raise HTTPException(404, "Song not found")
    return song

# --- SEARCH ---
@router.get("/search", response_model=Dict[str, Any])
def search_library(q: str = "", limit: int = 50, session: Session = Depends(get_session)):
    """
    Optimized smart search using SQL-side sorting (CASE statements).
    """
    if not q:
        return {"songs": [], "albums": [], "artists": [], "bestMatchType": None}
    
    q_lower = q.lower().strip()
    search_term = f"%{q}%"
    start_term = f"{q}%"
    
    # === SONGS ===
    song_score = case(
        (func.lower(Song.title) == q_lower, 30),
        (Song.title.ilike(start_term), 20),
        (Song.title.ilike(search_term), 10),
        else_=0
    ) + case(
        (func.lower(Song.artist) == q_lower, 5),
        (Song.artist.ilike(start_term), 3),
        (Song.artist.ilike(search_term), 1),
        else_=0
    )
    
    songs = session.exec(
        select(Song)
        .where(
            or_(
                Song.title.ilike(search_term),
                Song.artist.ilike(search_term)
            )
        )
        .order_by(song_score.desc())
        .limit(limit)
    ).all()
    
    # === ALBUMS ===
    album_score = case(
        (func.lower(Album.title) == q_lower, 30),
        (Album.title.ilike(start_term), 20),
        (Album.title.ilike(search_term), 10),
        else_=0
    ) + case(
        (func.lower(Album.artist) == q_lower, 5),
        (Album.artist.ilike(start_term), 3),
        (Album.artist.ilike(search_term), 1),
        else_=0
    )
    
    albums = session.exec(
        select(Album)
        .where(
            or_(
                Album.title.ilike(search_term),
                Album.artist.ilike(search_term)
            )
        )
        .order_by(album_score.desc())
        .limit(20)
    ).all()
    
    # === ARTISTS ===
    raw_artist_matches = session.exec(
        select(Song.artist).where(Song.artist.ilike(search_term)).distinct()
    ).all()
    
    found_artists = set()
    for art_str in raw_artist_matches:
        if not art_str: continue
        parts = re.split(r'\s*[,&]\s*|\s+feat\.?\s+|\s+ft\.?\s+|\s+featuring\s+|\s+and\s+|\s+with\s+', art_str, flags=re.IGNORECASE)
        for p in parts:
            p_clean = p.strip()
            if q_lower in p_clean.lower():
                found_artists.add(p_clean)
                
    sorted_artists = sorted(list(found_artists), key=lambda x: (x.lower() == q_lower, x.lower().startswith(q_lower)), reverse=True)
    artists = [{"name": name} for name in sorted_artists[:10]]

    # Best Match Determination
    best_match_type = None
    
    song_exact = songs[0].title.lower() == q_lower if songs else False
    album_exact = albums[0].title.lower() == q_lower if albums else False
    artist_exact = sorted_artists[0].lower() == q_lower if sorted_artists else False
    
    if song_exact:
        best_match_type = "song"
    elif album_exact:
        best_match_type = "album"
    elif artist_exact:
        best_match_type = "artist"
    elif songs: 
        best_match_type = "song"

    return {
        "songs": songs,
        "albums": albums,
        "artists": artists,
        "bestMatchType": best_match_type
    }

# --- ALBUMS ---
@router.get("/albums", response_model=List[AlbumRead])
def get_albums(offset: int = 0, limit: int = 50, session: Session = Depends(get_session)):
    # Efficiently count songs per album using a subquery or join
    # 1. Subquery to count songs per album
    sq = select(Song.album_id, func.count(Song.id).label("count")).group_by(Song.album_id).subquery()
    
    # 2. Join Album with Subquery
    query = (
        select(Album, func.coalesce(sq.c.count, 0).label("song_count"))
        .outerjoin(sq, Album.id == sq.c.album_id)
        .offset(offset)
        .limit(limit)
    )
    
    results = session.exec(query).all()
    
    # 3. Construct AlbumRead objects
    albums_with_counts = []
    for album, count in results:
        # Create AlbumRead from Album data + extra field
        ar = AlbumRead.model_validate(album)
        ar.song_count = count
        albums_with_counts.append(ar)
        
    return albums_with_counts

@router.get("/albums/{album_id}", response_model=Album)
def get_album_details(album_id: int, session: Session = Depends(get_session)):
    return session.get(Album, album_id)

@router.get("/albums/{album_id}/songs", response_model=List[SongListItem])
def get_album_songs(album_id: int, session: Session = Depends(get_session)):
    return session.exec(select(Song).where(Song.album_id == album_id).order_by(Song.track_number, Song.title)).all()