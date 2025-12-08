import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface FilterState {
    // Songs page
    songsSortBy: string;
    songsSortOrder: 'asc' | 'desc';
    setSongsSort: (sortBy: string, order: 'asc' | 'desc') => void;

    // Albums page
    albumsSortBy: 'title' | 'artist' | 'year' | 'song_count';
    albumsSortOrder: 'asc' | 'desc';
    setAlbumsSort: (sortBy: 'title' | 'artist' | 'year' | 'song_count', order: 'asc' | 'desc') => void;

    // Artists page
    artistsSortBy: 'name' | 'album_count';
    artistsSortOrder: 'asc' | 'desc';
    setArtistsSort: (sortBy: 'name' | 'album_count', order: 'asc' | 'desc') => void;

    // Genres page
    genresSortBy: 'name' | 'song_count';
    genresSortOrder: 'asc' | 'desc';
    setGenresSort: (sortBy: 'name' | 'song_count', order: 'asc' | 'desc') => void;
}

export const useFilterStore = create<FilterState>()(
    persist(
        (set) => ({
            // Songs defaults
            songsSortBy: 'title',
            songsSortOrder: 'asc',
            setSongsSort: (sortBy, order) => set({ songsSortBy: sortBy, songsSortOrder: order }),

            // Albums defaults
            albumsSortBy: 'title',
            albumsSortOrder: 'asc',
            setAlbumsSort: (sortBy, order) => set({ albumsSortBy: sortBy, albumsSortOrder: order }),

            // Artists defaults
            artistsSortBy: 'name',
            artistsSortOrder: 'asc',
            setArtistsSort: (sortBy, order) => set({ artistsSortBy: sortBy, artistsSortOrder: order }),

            // Genres defaults
            genresSortBy: 'song_count',
            genresSortOrder: 'desc',
            setGenresSort: (sortBy, order) => set({ genresSortBy: sortBy, genresSortOrder: order }),
        }),
        {
            name: 'filter-storage',
        }
    )
);
