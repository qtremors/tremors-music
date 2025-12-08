import { useState, useRef, useEffect, useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { getAlbums, getCoverUrl, getAlbumSongs, getSongs } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { useToastStore } from '../stores/toastStore';
import { Disc, MoreVertical, Play, ListPlus, User, ArrowUpDown } from 'lucide-react';
import { AlbumGridSkeleton } from '../components/Skeletons';
import { ArtistLink } from '../components/ContextMenu';
import { useFilterStore } from '../stores/filterStore';
import { Album } from '../types';

type SortField = 'title' | 'artist' | 'year' | 'song_count';
type SortOrder = 'asc' | 'desc';

// --- Album Card with Context Menu ---
function AlbumCard({ album }: { album: Album }) {
  const navigate = useNavigate();
  const [imageError, setImageError] = useState(false);
  const [showMenu, setShowMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const { playSong, setQueue, addToQueue } = usePlayerStore();
  const { addToast } = useToastStore();

  // Close menu on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setShowMenu(false);
      }
    };
    if (showMenu) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [showMenu]);

  const handlePlayAlbum = async (e?: React.MouseEvent) => {
    e?.stopPropagation();
    try {
      const songs = await getAlbumSongs(album.id);
      if (songs.length > 0) {
        setQueue(songs);
        playSong(songs[0]);
      }
    } catch (err) {
      addToast('Failed to play album', 'error');
    }
    setShowMenu(false);
  };

  const handleAddToQueue = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      const songs = await getAlbumSongs(album.id);
      songs.forEach(song => addToQueue(song));
      addToast(`Added ${songs.length} songs to queue`);
    } catch (err) {
      addToast('Failed to add to queue', 'error');
    }
    setShowMenu(false);
  };

  const handleGoToArtist = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (album.artist) {
      navigate(`/artists/${encodeURIComponent(album.artist)}`);
    }
    setShowMenu(false);
  };

  return (
    <div
      className="group cursor-pointer relative"
      onClick={() => navigate(`/albums/${album.id}`)}
    >
      {/* Album Art */}
      <div className="aspect-square bg-gray-200 dark:bg-white/5 rounded-lg shadow-sm overflow-hidden relative border border-black/5 dark:border-white/5 group-hover:shadow-md transition-all">
        {!imageError ? (
          <img
            src={getCoverUrl(album.id)}
            alt={album.title}
            className="w-full h-full object-cover transition duration-500 group-hover:scale-105"
            loading="lazy"
            onError={() => setImageError(true)}
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-gray-400 bg-gray-100 dark:bg-white/5">
            <Disc size={40} />
          </div>
        )}

        {/* Hover overlay with play button */}
        <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
          <button
            onClick={handlePlayAlbum}
            className="w-12 h-12 rounded-full bg-apple-accent text-white flex items-center justify-center shadow-lg transform scale-90 group-hover:scale-100 transition"
          >
            <Play size={20} fill="white" className="ml-0.5" />
          </button>
        </div>

        {/* 3-dot menu */}
        <div ref={menuRef} className="absolute top-2 right-2">
          <button
            onClick={(e) => { e.stopPropagation(); setShowMenu(!showMenu); }}
            className="p-1.5 rounded-full bg-black/50 text-white opacity-0 group-hover:opacity-100 transition hover:bg-black/70"
          >
            <MoreVertical size={16} />
          </button>

          {showMenu && (
            <div
              className="absolute right-0 top-8 w-44 bg-white dark:bg-[#2c2c2e] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 py-1.5 z-50 animate-in fade-in zoom-in-95 duration-100"
              onClick={(e) => e.stopPropagation()}
            >
              <button
                onClick={handlePlayAlbum}
                className="w-full flex items-center gap-3 px-3 py-2 text-sm text-apple-text hover:bg-gray-100 dark:hover:bg-white/10 transition"
              >
                <Play size={14} className="text-apple-subtext" />
                Play Album
              </button>
              <button
                onClick={handleAddToQueue}
                className="w-full flex items-center gap-3 px-3 py-2 text-sm text-apple-text hover:bg-gray-100 dark:hover:bg-white/10 transition"
              >
                <ListPlus size={14} className="text-apple-subtext" />
                Add to Queue
              </button>
              <div className="h-px bg-gray-100 dark:bg-white/10 my-1" />
              <button
                onClick={handleGoToArtist}
                className="w-full flex items-center gap-3 px-3 py-2 text-sm text-apple-text hover:bg-gray-100 dark:hover:bg-white/10 transition"
              >
                <User size={14} className="text-apple-subtext" />
                Go to Artist
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Album Info */}
      <div className="mt-3">
        <h3 className="text-sm font-medium text-apple-text truncate group-hover:text-apple-accent transition-colors">
          {album.title}
        </h3>
        <p className="text-xs text-apple-subtext truncate">
          <ArtistLink name={album.artist} />
        </p>
      </div>
    </div>
  );
}

// --- Main Page ---
export function AlbumsPage() {
  const { albumsSortBy: sortBy, albumsSortOrder: sortOrder, setAlbumsSort } = useFilterStore();
  const [showSortMenu, setShowSortMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  const { data: albums, isLoading } = useQuery({
    queryKey: ['albums'],
    queryFn: () => getAlbums(0, 1000),
  });

  // Fetch all songs to count per album
  const { data: allSongs } = useQuery({
    queryKey: ['all-songs-for-count'],
    queryFn: () => getSongs(0, 10000, 'title', 'asc'),
  });

  // Create album song count map
  const albumSongCount = useMemo(() => {
    const counts: Record<number, number> = {};
    allSongs?.forEach(song => {
      if (song.album_id) {
        counts[song.album_id] = (counts[song.album_id] || 0) + 1;
      }
    });
    return counts;
  }, [allSongs]);

  // Close menu on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setShowSortMenu(false);
      }
    };
    if (showSortMenu) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [showSortMenu]);

  // Sort albums
  const sortedAlbums = albums ? [...albums].sort((a, b) => {
    let aVal: string | number;
    let bVal: string | number;

    // Use computed song count for song_count sorting
    if (sortBy === 'song_count') {
      aVal = albumSongCount[a.id] || 0;
      bVal = albumSongCount[b.id] || 0;
    } else if (sortBy === 'year') {
      aVal = Number(a.year) || 0;
      bVal = Number(b.year) || 0;
    } else {
      aVal = (a[sortBy] ?? '').toString().toLowerCase();
      bVal = (b[sortBy] ?? '').toString().toLowerCase();
    }

    if (sortOrder === 'asc') {
      return aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
    } else {
      return aVal > bVal ? -1 : aVal < bVal ? 1 : 0;
    }
  }) : [];

  const handleSort = (field: SortField) => {
    if (sortBy === field) {
      setAlbumsSort(sortBy, sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setAlbumsSort(field, 'asc');
    }
    setShowSortMenu(false);
  };

  if (isLoading) return (
    <div className="h-full flex flex-col bg-apple-gray">
      <header className="h-16 flex items-center px-8 sticky top-0 z-10 border-b border-gray-200 dark:border-white/10 bg-apple-gray/95 backdrop-blur flex-shrink-0">
        <h2 className="text-xl font-semibold text-apple-text">Albums</h2>
      </header>
      <AlbumGridSkeleton count={18} />
    </div>
  );

  return (
    <div className="h-full flex flex-col bg-apple-gray">
      <header className="h-16 flex items-center justify-between px-8 sticky top-0 z-10 border-b border-gray-200 dark:border-white/10 bg-apple-gray/95 backdrop-blur flex-shrink-0">
        <h2 className="text-xl font-semibold text-apple-text">Albums</h2>

        {/* Sort dropdown */}
        <div ref={menuRef} className="relative">
          <button
            onClick={() => setShowSortMenu(!showSortMenu)}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm text-apple-subtext hover:text-apple-text hover:bg-gray-200 dark:hover:bg-white/10 transition"
          >
            <ArrowUpDown size={14} />
            <span className="hidden sm:inline">Sort</span>
          </button>

          {showSortMenu && (
            <div className="absolute right-0 top-10 w-44 bg-white dark:bg-[#2c2c2e] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 py-1.5 z-50 animate-in fade-in zoom-in-95 duration-100">
              <SortOption label="Title" field="title" currentSort={sortBy} sortOrder={sortOrder} onClick={handleSort} />
              <SortOption label="Artist" field="artist" currentSort={sortBy} sortOrder={sortOrder} onClick={handleSort} />
              <SortOption label="Year" field="year" currentSort={sortBy} sortOrder={sortOrder} onClick={handleSort} />
              <SortOption label="Song Count" field="song_count" currentSort={sortBy} sortOrder={sortOrder} onClick={handleSort} />
            </div>
          )}
        </div>
      </header>

      <main className="flex-1 overflow-y-auto p-8 pb-32">
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6">
          {sortedAlbums.map((album) => (
            <AlbumCard key={album.id} album={album} />
          ))}
        </div>
      </main>
    </div>
  );
}

// Sort option helper
function SortOption({
  label,
  field,
  currentSort,
  sortOrder,
  onClick
}: {
  label: string;
  field: SortField;
  currentSort: SortField;
  sortOrder: SortOrder;
  onClick: (field: SortField) => void;
}) {
  const isActive = currentSort === field;
  return (
    <button
      onClick={() => onClick(field)}
      className="w-full flex items-center justify-between px-3 py-2 text-sm hover:bg-gray-100 dark:hover:bg-white/10 transition"
    >
      <span className={isActive ? 'text-apple-accent font-medium' : 'text-apple-text'}>{label}</span>
      {isActive && (
        <span className="text-apple-accent text-xs">
          {sortOrder === 'asc' ? '↑' : '↓'}
        </span>
      )}
    </button>
  );
}