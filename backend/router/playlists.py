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