export interface Album {
  id: number;
  title: string;
  artist: string;
  cover_path?: string;

  // Expanded metadata
  year?: number;
  genre?: string;
  total_tracks?: number;
  total_discs?: number;
  compilation?: boolean;
  label?: string;
  barcode?: string;
  catalog_number?: string;
  song_count?: number;
}

export interface Song {
  id: number;

  // Basic information
  title: string;
  artist: string;
  album_id?: number;
  path: string;

  // People & credits
  composer?: string;
  conductor?: string;
  lyricist?: string;
  arranger?: string;
  performer?: string;
  remixer?: string;
  engineer?: string;
  producer?: string;

  // Organization & cataloging
  track_number?: number;
  disc_number?: number;
  genre?: string;
  compilation?: boolean;
  isrc?: string;

  // Dates
  year?: number;
  release_date?: string;
  original_date?: string;

  // Technical audio info
  duration: number;
  file_size: number;
  bitrate?: number;
  sample_rate?: number;
  channels?: number;
  bits_per_sample?: number;
  format: string;
  codec?: string;

  // Content & description
  has_lyrics: boolean;
  lyrics?: string;
  comment?: string;
  description?: string;
  language?: string;
  mood?: string;

  // Musical information
  bpm?: number;
  initial_key?: string;

  // ReplayGain
  replaygain_track_gain?: number;
  replaygain_track_peak?: number;
  replaygain_album_gain?: number;
  replaygain_album_peak?: number;

  // User data
  rating?: number;
  play_count?: number;
  last_played?: string;
  date_added?: string;

  // Media type
  media_type?: string;
  grouping?: string;
  subtitle?: string;

  // Relationship
  album?: Album;
}

export interface Artist {
  name: string;
  album_count: number;
  cover_album_id?: number;
  cover_example?: number; // Used for display
}

export interface Genre {
  name: string;
  song_count: number;
}