export interface Album {
  id: number;
  title: string;
  artist: string;
  cover_path?: string;
}

export interface Song {
  id: number;
  title: string;
  artist: string;
  album_id?: number;
  duration: number;
  path: string;
  has_lyrics: boolean;
  track_number?: number;
  album?: Album;
}

export interface Album {
  id: number;
  title: string;
  artist: string;
  cover_path?: string;
}

export interface Song {
  id: number;
  title: string;
  artist: string;
  album_id?: number;
  duration: number;
  path: string;
  has_lyrics: boolean;
  track_number?: number;
  
  // Tech Specs
  file_size: number;
  bitrate?: number;
  sample_rate?: number;
  format: string;

  album?: Album;
}