import axios from 'axios';
import { Song, Album } from '../types';

const api = axios.create({
  baseURL: '/api', 
});

// --- SONGS ---
export const getSongs = async (offset = 0, limit = 5000, sortBy = 'title', order = 'asc') => {
  const response = await api.get<Song[]>(`/library/songs`, {
    params: { offset, limit, sort_by: sortBy, order }
  });
  return response.data;
};

export const searchLibrary = async (query: string) => {
  if (!query) return [];
  const response = await api.get<Song[]>(`/library/search?q=${encodeURIComponent(query)}`);
  return response.data;
};

// --- ALBUMS ---
export const getAlbums = async (offset = 0, limit = 50) => {
  const response = await api.get<Album[]>(`/library/albums?offset=${offset}&limit=${limit}`);
  return response.data;
};

export const getAlbum = async (id: string) => {
  const response = await api.get<Album>(`/library/albums/${id}`);
  return response.data;
};

export const getAlbumSongs = async (id: string) => {
  const response = await api.get<Song[]>(`/library/albums/${id}/songs`);
  return response.data;
};

export const getCoverUrl = (albumId: number) => {
  return `http://127.0.0.1:8000/covers/${albumId}`;
};

// --- ARTISTS ---
export interface Artist {
  name: string;
  album_count: number;
  cover_example: number;
}

export interface ArtistWork {
  album: Album | null;
  songs: Song[];
}

export const getArtists = async () => {
  const response = await api.get<Artist[]>('/library/artists');
  return response.data;
};

export const getArtistWork = async (name: string) => {
  const response = await api.get<ArtistWork[]>(`/library/artists/${encodeURIComponent(name)}/work`);
  return response.data;
};

// --- PLAYLISTS ---
export interface Playlist {
  id: number;
  name: string;
}

export const getPlaylists = async () => {
  const response = await api.get<Playlist[]>('/playlists');
  return response.data;
};

export const createPlaylist = async (name: string) => {
  const response = await api.post('/playlists', { name });
  return response.data;
};

export const addToPlaylist = async (playlistId: number, songIds: number[]) => {
  await api.post(`/playlists/${playlistId}/add`, { song_ids: songIds });
};

export const getPlaylistSongs = async (id: string) => {
  const response = await api.get<Song[]>(`/playlists/${id}/songs`);
  return response.data;
};

export const deletePlaylist = async (id: number) => {
  await api.delete(`/playlists/${id}`);
};

export const renamePlaylist = async (id: number, name: string) => {
  const response = await api.patch<Playlist>(`/playlists/${id}`, { name });
  return response.data;
};

// --- SYSTEM ---
export const scanLibrary = async (path: string) => {
  const response = await api.post('/library/scan', { path });
  return response.data;
};

export const resetLibrary = async () => {
  const response = await api.delete('/library/reset');
  return response.data;
};

export const getLyrics = async (songId: number) => {
  const response = await api.get(`/lyrics/${songId}`);
  return response.data;
};

export const getStreamUrl = (songId: number) => {
  return `http://127.0.0.1:8000/stream/${songId}`;
};

export default api;