from fastapi import APIRouter, Depends, HTTPException
from sqlmodel import SQLModel, Session, select
from database import get_session
from models import Playlist, PlaylistSong, Song
from typing import List

router = APIRouter(prefix="/playlists", tags=["Playlists"])

class PlaylistCreate(SQLModel):
    name: str

class PlaylistAdd(SQLModel):
    song_ids: List[int]

class PlaylistUpdate(SQLModel):
    name: str

class PlaylistReorder(SQLModel):
    song_ids: List[int]

@router.post("/", response_model=Playlist)
def create_playlist(payload: PlaylistCreate, session: Session = Depends(get_session)):
    pl = Playlist(name=payload.name)
    session.add(pl)
    session.commit()
    session.refresh(pl)
    return pl

@router.get("/", response_model=List[Playlist])
def get_playlists(session: Session = Depends(get_session)):
    return session.exec(select(Playlist)).all()

@router.get("/{playlist_id}", response_model=Playlist)
def get_playlist(playlist_id: int, session: Session = Depends(get_session)):
    pl = session.get(Playlist, playlist_id)
    if not pl: raise HTTPException(404, "Playlist not found")
    return pl

@router.patch("/{playlist_id}", response_model=Playlist)
def update_playlist(playlist_id: int, payload: PlaylistUpdate, session: Session = Depends(get_session)):
    pl = session.get(Playlist, playlist_id)
    if not pl: raise HTTPException(404)
    pl.name = payload.name
    session.add(pl)
    session.commit()
    session.refresh(pl)
    return pl

@router.get("/{playlist_id}/songs", response_model=List[Song])
def get_playlist_songs(playlist_id: int, session: Session = Depends(get_session)):
    stmt = (
        select(Song)
        .join(PlaylistSong)
        .where(PlaylistSong.playlist_id == playlist_id)
        .order_by(PlaylistSong.order)
    )
    return session.exec(stmt).all()

@router.post("/{playlist_id}/add")
def add_songs_to_playlist(playlist_id: int, payload: PlaylistAdd, session: Session = Depends(get_session)):
    pl = session.get(Playlist, playlist_id)
    if not pl: raise HTTPException(404)
    current_count = len(session.exec(select(PlaylistSong).where(PlaylistSong.playlist_id == playlist_id)).all())
    for i, song_id in enumerate(payload.song_ids):
        # We allow duplicates as per standard playlist behavior, but my reorder logic assumes we might want to be careful.
        # For now, just add them.
        link = PlaylistSong(playlist_id=playlist_id, song_id=song_id, order=current_count + i)
        session.add(link)
    session.commit()
    return {"message": "Songs added"}

@router.delete("/{playlist_id}")
def delete_playlist(playlist_id: int, session: Session = Depends(get_session)):
    pl = session.get(Playlist, playlist_id)
    if not pl: raise HTTPException(404)
    links = session.exec(select(PlaylistSong).where(PlaylistSong.playlist_id == playlist_id)).all()
    for link in links: session.delete(link)
    session.delete(pl)
    session.commit()
    return {"message": "Deleted"}

@router.delete("/{playlist_id}/songs/{song_id}")
def remove_song_from_playlist(playlist_id: int, song_id: int, session: Session = Depends(get_session)):
    """Remove a song from a playlist."""
    # Removes ALL instances of this song in the playlist for now to simplify
    links = session.exec(
        select(PlaylistSong)
        .where(PlaylistSong.playlist_id == playlist_id)
        .where(PlaylistSong.song_id == song_id)
    ).all()
    
    if not links:
        raise HTTPException(status_code=404, detail="Song not found in playlist")
        
    for link in links:
        session.delete(link)
    session.commit()
    return {"message": "Song removed"}

@router.post("/{playlist_id}/reorder")
def reorder_playlist(playlist_id: int, payload: PlaylistReorder, session: Session = Depends(get_session)):
    """Reorder playlist based on a list of song IDs."""
    pl = session.get(Playlist, playlist_id)
    if not pl: raise HTTPException(404, "Playlist not found")
    
    links = session.exec(
        select(PlaylistSong).where(PlaylistSong.playlist_id == playlist_id)
    ).all()
    
    # Map song_id to list of links to handle duplicates
    link_map = {}
    for link in links:
        if link.song_id not in link_map:
            link_map[link.song_id] = []
        link_map[link.song_id].append(link)
    
    current_order = 0
    for song_id in payload.song_ids:
        if song_id in link_map and link_map[song_id]:
            link = link_map[song_id].pop(0)
            link.order = current_order
            session.add(link)
            current_order += 1
            
    session.commit()
    return {"message": "Playlist reordered"}