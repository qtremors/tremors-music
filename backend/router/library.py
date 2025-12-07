from typing import List, Dict, Any
from fastapi import APIRouter, Depends, BackgroundTasks, HTTPException
from sqlmodel import Session, select, func, or_
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
@router.get("/search", response_model=List[Song])
def search_songs(
    q: str = "",
    year_min: int | None = None,
    year_max: int | None = None,
    bpm_min: int | None = None,
    bpm_max: int | None = None,
    rating_min: int | None = None,
    genre: str | None = None,
    duration_min: int | None = None,
    duration_max: int | None = None,
    format: str | None = None,
    limit: int = 100,
    session: Session = Depends(get_session)
):
    """
    Advanced search with text query and filters.
    
    - q: Search in title, artist, album, composer
    - year_min/max: Filter by year range
    - bpm_min/max: Filter by BPM range
    - rating_min: Minimum rating (1-5)
    - genre: Filter by genre (exact match)
    - duration_min/max: Filter by duration in seconds
    - format: Filter by file format (mp3, flac, etc.)
    - limit: Maximum results (default 100)
    """
    query = select(Song)
    
    # Text search across multiple fields
    if q:
        search_term = f"%{q}%"
        query = query.where(
            or_(
                Song.title.ilike(search_term),
                Song.artist.ilike(search_term),
                Song.album_title.ilike(search_term),
                Song.composer.ilike(search_term)
            )
        )
    
    # Year filter
    if year_min is not None:
        query = query.where(Song.year >= year_min)
    if year_max is not None:
        query = query.where(Song.year <= year_max)
    
    # BPM filter
    if bpm_min is not None:
        query = query.where(Song.bpm >= bpm_min)
    if bpm_max is not None:
        query = query.where(Song.bpm <= bpm_max)
    
    # Rating filter
    if rating_min is not None:
        query = query.where(Song.rating >= rating_min)
    
    # Genre filter
    if genre:
        query = query.where(Song.genre.ilike(f"%{genre}%"))
    
    # Duration filter
    if duration_min is not None:
        query = query.where(Song.duration >= duration_min)
    if duration_max is not None:
        query = query.where(Song.duration <= duration_max)
    
    # Format filter
    if format:
        query = query.where(Song.format.ilike(f"%{format}%"))
    
    # Apply limit and execute
    query = query.limit(limit)
    results = session.exec(query).all()
    
    return results

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