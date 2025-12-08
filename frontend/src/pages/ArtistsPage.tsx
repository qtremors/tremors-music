import { useState, useRef, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { getArtists, getCoverUrl, getArtistWork } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { useToastStore } from '../stores/toastStore';
import { Mic2, MoreVertical, Play, ListPlus, ArrowUpDown, Shuffle } from 'lucide-react';
import { ArtistGridSkeleton } from '../components/Skeletons';
import { useFilterStore } from '../stores/filterStore';
import { Artist } from '../types';

type SortField = 'name' | 'album_count';
type SortOrder = 'asc' | 'desc';

// --- Artist Card with Context Menu ---
function ArtistCard({ artist }: { artist: Artist }) {
  const navigate = useNavigate();
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

  const handlePlayArtist = async (e?: React.MouseEvent) => {
    e?.stopPropagation();
    try {
      const workItems = await getArtistWork(artist.name);
      // Flatten all songs from all work items
      const allSongs = workItems.flatMap(w => w.songs);
      if (allSongs.length > 0) {
        setQueue(allSongs);
        playSong(allSongs[0]);
      }
    } catch {
      addToast('Failed to play artist', 'error');
    }
    setShowMenu(false);
  };

  const handleShuffleArtist = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      const workItems = await getArtistWork(artist.name);
      const allSongs = workItems.flatMap(w => w.songs);
      if (allSongs.length > 0) {
        // Fisher-Yates shuffle
        const shuffled = [...allSongs];
        for (let i = shuffled.length - 1; i > 0; i--) {
          const j = Math.floor(Math.random() * (i + 1));
          [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
        }
        setQueue(shuffled);
        usePlayerStore.setState({ isShuffle: true, originalQueue: allSongs });
        playSong(shuffled[0]);
      }
    } catch {
      addToast('Failed to shuffle artist', 'error');
    }
    setShowMenu(false);
  };

  const handleAddToQueue = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      const workItems = await getArtistWork(artist.name);
      const allSongs = workItems.flatMap(w => w.songs);
      if (allSongs.length > 0) {
        allSongs.forEach(song => addToQueue(song));
        addToast(`Added ${allSongs.length} songs to queue`);
      }
    } catch {
      addToast('Failed to add to queue', 'error');
    }
    setShowMenu(false);
  };

  return (
    <div
      className="group cursor-pointer text-center space-y-3 relative"
      onClick={() => navigate(`/artists/${encodeURIComponent(artist.name)}`)}
    >
      {/* Artist Image */}
      <div className="aspect-square rounded-full overflow-hidden bg-gray-200 dark:bg-white/5 shadow-md border border-white/10 mx-auto w-full max-w-[200px] relative group-hover:shadow-lg transition-all">
        <img
          src={getCoverUrl(artist.cover_example ?? 0)}
          className="w-full h-full object-cover group-hover:scale-110 transition duration-500"
          onError={(e) => e.currentTarget.style.display = 'none'}
        />
        <div className="absolute inset-0 flex items-center justify-center -z-10">
          <Mic2 size={40} className="text-gray-400" />
        </div>

        {/* Hover overlay with play button */}
        <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center rounded-full">
          <button
            onClick={handlePlayArtist}
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
                onClick={handlePlayArtist}
                className="w-full flex items-center gap-3 px-3 py-2 text-sm text-apple-text hover:bg-gray-100 dark:hover:bg-white/10 transition"
              >
                <Play size={14} className="text-apple-subtext" />
                Play All
              </button>
              <button
                onClick={handleShuffleArtist}
                className="w-full flex items-center gap-3 px-3 py-2 text-sm text-apple-text hover:bg-gray-100 dark:hover:bg-white/10 transition"
              >
                <Shuffle size={14} className="text-apple-subtext" />
                Shuffle All
              </button>
              <button
                onClick={handleAddToQueue}
                className="w-full flex items-center gap-3 px-3 py-2 text-sm text-apple-text hover:bg-gray-100 dark:hover:bg-white/10 transition"
              >
                <ListPlus size={14} className="text-apple-subtext" />
                Add to Queue
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Artist Info */}
      <div>
        <h3 className="font-medium text-apple-text truncate group-hover:text-apple-accent transition">{artist.name}</h3>
        <p className="text-xs text-apple-subtext">{artist.album_count} Albums</p>
      </div>
    </div>
  );
}

// --- Main Page ---
export function ArtistsPage() {
  const { artistsSortBy: sortBy, artistsSortOrder: sortOrder, setArtistsSort } = useFilterStore();
  const [showSortMenu, setShowSortMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  const { data: artists, isLoading } = useQuery({
    queryKey: ['artists'],
    queryFn: getArtists,
  });

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

  // Sort artists
  const sortedArtists = artists ? [...artists].sort((a, b) => {
    let aVal = a[sortBy];
    let bVal = b[sortBy];

    if (typeof aVal === 'string') aVal = aVal.toLowerCase();
    if (typeof bVal === 'string') bVal = bVal.toLowerCase();

    if (sortOrder === 'asc') {
      return aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
    } else {
      return aVal > bVal ? -1 : aVal < bVal ? 1 : 0;
    }
  }) : [];

  const handleSort = (field: SortField) => {
    if (sortBy === field) {
      setArtistsSort(sortBy, sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setArtistsSort(field, 'asc');
    }
    setShowSortMenu(false);
  };

  if (isLoading) return (
    <div className="h-full flex flex-col">
      <header className="h-16 flex items-center px-8 border-b border-gray-200 dark:border-white/10 bg-apple-gray/95 backdrop-blur z-10">
        <h2 className="text-xl font-semibold text-apple-text">Artists</h2>
      </header>
      <ArtistGridSkeleton count={15} />
    </div>
  );

  return (
    <div className="h-full flex flex-col bg-apple-gray">
      <header className="h-16 flex items-center justify-between px-8 border-b border-gray-200 dark:border-white/10 bg-apple-gray/95 backdrop-blur z-10">
        <h2 className="text-xl font-semibold text-apple-text">Artists</h2>

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
            <div className="absolute right-0 top-10 w-40 bg-white dark:bg-[#2c2c2e] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 py-1.5 z-50 animate-in fade-in zoom-in-95 duration-100">
              <SortOption label="Name" field="name" currentSort={sortBy} sortOrder={sortOrder} onClick={handleSort} />
              <SortOption label="Album Count" field="album_count" currentSort={sortBy} sortOrder={sortOrder} onClick={handleSort} />
            </div>
          )}
        </div>
      </header>

      <main className="flex-1 overflow-y-auto p-8 pb-32">
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-6">
          {sortedArtists.map((artist) => (
            <ArtistCard key={artist.name} artist={artist} />
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