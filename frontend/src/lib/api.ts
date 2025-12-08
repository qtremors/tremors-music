import axios from 'axios';
import { Song, Album } from '../types';

// In development, Vite proxies /api to the backend.
// In production (Tauri build), we need the full backend URL.
// We allow configuration via VITE_API_PORT for custom builds.
const PORT = import.meta.env.VITE_API_PORT || '8000';
const API_BASE = import.meta.env.DEV ? '/api' : `http://127.0.0.1:${PORT}`;

const api = axios.create({
  baseURL: API_BASE,
});

// --- SONGS ---
export const getSongs = async (offset = 0, limit = 5000, sortBy = 'title', order = 'asc') => {
  const response = await api.get<Song[]>(`/library/songs`, {
    params: { offset, limit, sort_by: sortBy, order }
  });
  return Array.isArray(response.data) ? response.data : [];
};

export const getSong = async (id: number) => {
  const response = await api.get<Song>(`/library/songs/${id}`);
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
  return Array.isArray(response.data) ? response.data : [];
};

export const getAlbum = async (id: number) => {
  const response = await api.get<Album>(`/library/albums/${id}`);
  return response.data;
};

export const getAlbumSongs = async (id: number) => {
  const response = await api.get<Song[]>(`/library/albums/${id}/songs`);
  return Array.isArray(response.data) ? response.data : [];
};

export const getCoverUrl = (albumId: number) => {
  return `${API_BASE}/covers/${albumId}?v=2`;
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
  return Array.isArray(response.data) ? response.data : [];
};

export const getArtistWork = async (name: string) => {
  const response = await api.get<ArtistWork[]>(`/library/artists/${encodeURIComponent(name)}/work`);
  return Array.isArray(response.data) ? response.data : [];
};

// --- GENRES ---
export interface Genre {
  name: string;
  song_count: number;
}

export const getGenres = async () => {
  const response = await api.get<Genre[]>('/library/genres');
  return Array.isArray(response.data) ? response.data : [];
};

export const getGenreSongs = async (name: string) => {
  // Use query parameter to avoid issues with / in genre names like "R&B/Soul"
  const response = await api.get<Song[]>('/library/genres/songs', { params: { name } });
  return Array.isArray(response.data) ? response.data : [];
};

// --- Smart Playlists ---
export const getFavorites = async (limit = 100) => {
  const response = await api.get<Song[]>('/library/smart-playlists/favorites', { params: { limit } });
  return Array.isArray(response.data) ? response.data : [];
};

export const getRecentlyAdded = async (limit = 50) => {
  const response = await api.get<Song[]>('/library/smart-playlists/recently-added', { params: { limit } });
  return Array.isArray(response.data) ? response.data : [];
};

export const getMostPlayed = async (limit = 50) => {
  const response = await api.get<Song[]>('/library/smart-playlists/most-played', { params: { limit } });
  return Array.isArray(response.data) ? response.data : [];
};

export const incrementPlayCount = async (songId: number) => {
  const response = await api.post<{ play_count: number }>(`/library/songs/${songId}/play`);
  return response.data;
};

export const toggleFavorite = async (songId: number) => {
  const response = await api.post<{ rating: number; is_favorite: boolean }>(`/library/songs/${songId}/favorite`);
  return response.data;
};

export interface Playlist {
  id: number;
  name: string;
}

export const getPlaylists = async () => {
  const response = await api.get<Playlist[]>('/playlists');
  return Array.isArray(response.data) ? response.data : [];
};

export const createPlaylist = async (name: string) => {
  const response = await api.post('/playlists', { name });
  return response.data;
};

export const addToPlaylist = async (playlistId: number, songIds: number[]) => {
  await api.post(`/playlists/${playlistId}/add`, { song_ids: songIds });
};

export const removeSongFromPlaylist = async (playlistId: number, songId: number) => {
  await api.delete(`/playlists/${playlistId}/songs/${songId}`);
};

export const reorderPlaylist = async (playlistId: number, songIds: number[]) => {
  await api.post(`/playlists/${playlistId}/reorder`, { song_ids: songIds });
};

export const getPlaylistSongs = async (id: string) => {
  const response = await api.get<Song[]>(`/playlists/${id}/songs`);
  return Array.isArray(response.data) ? response.data : [];
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
  return `${API_BASE}/stream/${songId}`;
};

export default api;