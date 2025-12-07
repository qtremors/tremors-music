// Advanced search page with filters
import { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Search, SlidersHorizontal, X } from 'lucide-react';
import api from '../lib/api';
import { Song } from '../types';
import { Card } from '../components/common/Card';
import { usePlayerStore } from '../stores/playerStore';

interface SearchFilters {
  q: string;
  year_min?: number;
  year_max?: number;
  bpm_min?: number;
  bpm_max?: number;
  rating_min?: number;
  genre?: string;
  duration_min?: number;
  duration_max?: number;
  format?: string;
}

export function SearchPage() {
  const [filters, setFilters] = useState<SearchFilters>({ q: '' });
  const [showFilters, setShowFilters] = useState(false);
  const [debouncedFilters, setDebouncedFilters] = useState<SearchFilters>(filters);

  // Debounce search
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedFilters(filters);
    }, 300);
    return () => clearTimeout(timer);
  }, [filters]);

  const { data: results = [], isLoading } = useQuery({
    queryKey: ['search', debouncedFilters],
    queryFn: async () => {
      const params = new URLSearchParams();
      Object.entries(debouncedFilters).forEach(([key, value]) => {
        if (value !== undefined && value !== '') {
          params.append(key, String(value));
        }
      });
      const res = await api.get<Song[]>(`/library/search?${params.toString()}`);
      return res.data;
    },
    enabled: debouncedFilters.q.length > 0 || Object.keys(debouncedFilters).length > 1,
  });

  const hasActiveFilters = Object.entries(filters).some(
    ([key, value]) => key !== 'q' && value !== undefined && value !== ''
  );

  const clearFilters = () => {
    setFilters({ q: filters.q });
  };

  return (
    <div className="h-full flex flex-col overflow-hidden bg-apple-gray">
      {/* Header */}
      <div className="p-8 pb-4">
        <h1 className="text-3xl font-bold text-apple-text mb-6">Search</h1>

        {/* Search Bar */}
        <div className="relative">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-apple-subtext" size={20} />
          <input
            type="text"
            placeholder="Search songs, artists, albums, composers..."
            value={filters.q}
            onChange={(e) => setFilters({ ...filters, q: e.target.value })}
            className="w-full pl-12 pr-12 py-3 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-apple-text placeholder-apple-subtext focus:outline-none focus:ring-2 focus:ring-apple-accent"
          />
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`absolute right-4 top-1/2 -translate-y-1/2 p-2 rounded-lg transition ${showFilters || hasActiveFilters
              ? 'text-apple-accent bg-apple-accent/10'
              : 'text-apple-subtext hover:bg-gray-100 dark:hover:bg-white/5'
              }`}
          >
            <SlidersHorizontal size={18} />
          </button>
        </div>

        {/* Filters Panel */}
        {showFilters && (
          <Card className="mt-4">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-semibold text-apple-text uppercase tracking-wider">Filters</h3>
              {hasActiveFilters && (
                <button
                  onClick={clearFilters}
                  className="text-xs text-apple-accent hover:underline flex items-center gap-1"
                >
                  <X size={12} />
                  Clear Filters
                </button>
              )}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {/* Year Range */}
              <div>
                <label className="block text-xs font-medium text-apple-subtext mb-2">Year Range</label>
                <div className="flex gap-2">
                  <input
                    type="number"
                    placeholder="Min"
                    value={filters.year_min || ''}
                    onChange={(e) => setFilters({ ...filters, year_min: e.target.value ? parseInt(e.target.value) : undefined })}
                    className="w-full px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-sm text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
                  />
                  <input
                    type="number"
                    placeholder="Max"
                    value={filters.year_max || ''}
                    onChange={(e) => setFilters({ ...filters, year_max: e.target.value ? parseInt(e.target.value) : undefined })}
                    className="w-full px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-sm text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
                  />
                </div>
              </div>

              {/* BPM Range */}
              <div>
                <label className="block text-xs font-medium text-apple-subtext mb-2">BPM Range</label>
                <div className="flex gap-2">
                  <input
                    type="number"
                    placeholder="Min"
                    value={filters.bpm_min || ''}
                    onChange={(e) => setFilters({ ...filters, bpm_min: e.target.value ? parseInt(e.target.value) : undefined })}
                    className="w-full px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-sm text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
                  />
                  <input
                    type="number"
                    placeholder="Max"
                    value={filters.bpm_max || ''}
                    onChange={(e) => setFilters({ ...filters, bpm_max: e.target.value ? parseInt(e.target.value) : undefined })}
                    className="w-full px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-sm text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
                  />
                </div>
              </div>

              {/* Rating */}
              <div>
                <label className="block text-xs font-medium text-apple-subtext mb-2">Minimum Rating</label>
                <select
                  value={filters.rating_min || ''}
                  onChange={(e) => setFilters({ ...filters, rating_min: e.target.value ? parseInt(e.target.value) : undefined })}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-sm text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
                >
                  <option value="">Any</option>
                  <option value="1">★ and above</option>
                  <option value="2">★★ and above</option>
                  <option value="3">★★★ and above</option>
                  <option value="4">★★★★ and above</option>
                  <option value="5">★★★★★</option>
                </select>
              </div>

              {/* Genre */}
              <div>
                <label className="block text-xs font-medium text-apple-subtext mb-2">Genre</label>
                <input
                  type="text"
                  placeholder="e.g., Rock, Electronic"
                  value={filters.genre || ''}
                  onChange={(e) => setFilters({ ...filters, genre: e.target.value || undefined })}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-sm text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
                />
              </div>

              {/* Format */}
              <div>
                <label className="block text-xs font-medium text-apple-subtext mb-2">Format</label>
                <select
                  value={filters.format || ''}
                  onChange={(e) => setFilters({ ...filters, format: e.target.value || undefined })}
                  className="w-full px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-sm text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
                >
                  <option value="">Any</option>
                  <option value="mp3">MP3</option>
                  <option value="flac">FLAC</option>
                  <option value="m4a">M4A</option>
                  <option value="wav">WAV</option>
                  <option value="ogg">OGG</option>
                </select>
              </div>

              {/* Duration */}
              <div>
                <label className="block text-xs font-medium text-apple-subtext mb-2">Duration (seconds)</label>
                <div className="flex gap-2">
                  <input
                    type="number"
                    placeholder="Min"
                    value={filters.duration_min || ''}
                    onChange={(e) => setFilters({ ...filters, duration_min: e.target.value ? parseInt(e.target.value) : undefined })}
                    className="w-full px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-sm text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
                  />
                  <input
                    type="number"
                    placeholder="Max"
                    value={filters.duration_max || ''}
                    onChange={(e) => setFilters({ ...filters, duration_max: e.target.value ? parseInt(e.target.value) : undefined })}
                    className="w-full px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 text-sm text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
                  />
                </div>
              </div>
            </div>
          </Card>
        )}
      </div>

      {/* Results */}
      <div className="flex-1 overflow-y-auto px-8 pb-32">
        {isLoading ? (
          <div className="flex items-center justify-center h-64">
            <div className="text-apple-subtext">Searching...</div>
          </div>
        ) : filters.q || hasActiveFilters ? (
          results.length > 0 ? (
            <>
              <div className="text-sm text-apple-subtext mb-4">
                Found {results.length} song{results.length !== 1 ? 's' : ''}
              </div>
              <div className="space-y-1">
                {results.map((song) => {
                  const isCurrent = usePlayerStore.getState().currentSong?.id === song.id;
                  return (
                    <div
                      key={song.id}
                      onClick={() => {
                        usePlayerStore.getState().setQueue(results);
                        usePlayerStore.getState().playSong(song);
                      }}
                      className={`flex items-center px-4 py-3 rounded-lg cursor-pointer group transition-colors hover:bg-gray-200/50 dark:hover:bg-white/5 ${isCurrent ? 'bg-gray-200 dark:bg-white/10' : ''
                        }`}
                    >
                      <div className="flex-1 min-w-0">
                        <div className={`text-sm font-medium truncate ${isCurrent ? 'text-apple-accent' : 'text-apple-text'}`}>
                          {song.title}
                        </div>
                        <div className="text-xs text-apple-subtext truncate">
                          {song.artist}
                        </div>
                      </div>
                      <div className="text-xs text-apple-subtext tabular-nums ml-4">
                        {Math.floor(song.duration / 60)}:{String(Math.floor(song.duration % 60)).padStart(2, '0')}
                      </div>
                    </div>
                  );
                })}
              </div>
            </>
          ) : (
            <div className="flex flex-col items-center justify-center h-64 text-center">
              <Search size={48} className="text-apple-subtext/30 mb-4" />
              <p className="text-apple-subtext">No results found</p>
              <p className="text-xs text-apple-subtext/70 mt-1">
                Try adjusting your search or filters
              </p>
            </div>
          )
        ) : (
          <div className="flex flex-col items-center justify-center h-64 text-center">
            <Search size={48} className="text-apple-subtext/30 mb-4" />
            <p className="text-apple-subtext">Start typing to search</p>
            <p className="text-xs text-apple-subtext/70 mt-1">
              Search by title, artist, album, or composer
            </p>
          </div>
        )}
      </div>
    </div>
  );
}