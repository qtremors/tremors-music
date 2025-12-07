import { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route, NavLink, useNavigate, useLocation } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { LibraryPage } from './pages/LibraryPage';
import { SearchPage } from './pages/SearchPage';
import { AlbumsPage } from './pages/AlbumsPage';
import { AlbumDetail } from './pages/AlbumDetail';
import { ArtistsPage } from './pages/ArtistsPage';
import { AllPlaylistsPage } from './pages/AllPlaylistsPage';
import { PlaylistDetail } from './pages/PlaylistDetail';
import { SettingsPage } from './pages/SettingsPage';
import { Player } from './components/Player';
import { ArtistDetail } from './pages/ArtistDetail';
import { ToastContainer } from './components/ToastContainer';
import { CreatePlaylistModal } from './components/CreatePlaylistModal';
import { getPlaylists } from './lib/api';
import { cn } from './lib/utils';
import {
  Library, Settings, Disc, Mic2, Music,
  PlusCircle, ListMusic, Search
} from 'lucide-react';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';

// --- Sidebar Component ---
// Handles navigation, playlists list, and global search input
function Sidebar({ setIsCreatingPlaylist }: { setIsCreatingPlaylist: (val: boolean) => void }) {
  const navigate = useNavigate();
  const location = useLocation();
  const [search, setSearch] = useState('');

  // Fetch Playlists for the sidebar list
  const { data: playlists } = useQuery({ queryKey: ['playlists'], queryFn: getPlaylists });

  // Debounced Search Logic
  useEffect(() => {
    const timer = setTimeout(() => {
      // Only navigate if there is a query or if we are already on the search page
      if (search.trim()) {
        navigate(`/search?q=${encodeURIComponent(search)}`);
      } else if (location.pathname === '/search' && !search.trim()) {
        navigate('/'); // Go home if search is cleared while on search page
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [search, navigate]);

  // Clear search input when navigating away from search page manually
  useEffect(() => {
    if (location.pathname !== '/search') {
      setSearch('');
    }
  }, [location.pathname]);

  return (
    <div className="fixed top-0 left-0 w-64 h-full pb-28 bg-gray-50/80 dark:bg-black/50 border-r border-gray-200 dark:border-white/10 hidden lg:flex flex-col p-6 z-20 backdrop-blur-xl">

      {/* Header & Search */}
      <div className="mb-6 px-2 flex-shrink-0">
        <h1 className="text-2xl font-bold text-apple-accent flex items-center gap-2 mb-6 select-none">
          <Library /> Tremors
        </h1>

        <div className="relative group">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-apple-subtext group-focus-within:text-apple-text transition-colors" />
          <input
            type="text"
            placeholder="Search Library"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-gray-200 dark:bg-white/10 rounded-lg pl-9 pr-3 py-2 text-sm text-apple-text outline-none focus:ring-2 focus:ring-apple-accent/50 transition-all placeholder:text-apple-subtext"
          />
        </div>
      </div>

      {/* Main Navigation Links */}
      <nav className="space-y-1 flex-shrink-0">
        <NavLink to="/" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
          <Music size={18} /> Songs
        </NavLink>
        <NavLink to="/albums" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
          <Disc size={18} /> Albums
        </NavLink>
        <NavLink to="/artists" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
          <Mic2 size={18} /> Artists
        </NavLink>
      </nav>

      {/* Playlists Header */}
      <div className="mt-8 mb-2 px-3 flex items-center justify-between group flex-shrink-0">
        <NavLink to="/playlists" className={({ isActive }) => cn("text-xs font-bold uppercase tracking-wider flex-1 transition", isActive ? "text-apple-accent" : "text-apple-subtext hover:text-apple-text")}>
          Playlists
        </NavLink>
        <button
          onClick={() => setIsCreatingPlaylist(true)}
          className="hover:text-apple-accent text-apple-subtext transition p-1 rounded hover:bg-black/5 dark:hover:bg-white/10"
          title="Create Playlist"
        >
          <PlusCircle size={16} />
        </button>
      </div>

      {/* Playlists Scrollable List */}
      <div className="space-y-1 overflow-y-auto flex-1 min-h-0 no-scrollbar -mx-2 px-2">
        {playlists?.map(pl => (
          <NavLink
            key={pl.id}
            to={`/playlists/${pl.id}`}
            className={({ isActive }) => cn(
              "w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition truncate text-sm",
              isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5"
            )}
          >
            <ListMusic size={16} className="flex-shrink-0" />
            <span className="truncate">{pl.name}</span>
          </NavLink>
        ))}
        {(!playlists || playlists.length === 0) && (
          <div className="px-3 py-4 text-xs text-apple-subtext text-center italic">
            No playlists yet
          </div>
        )}
      </div>

      {/* Settings Link (Bottom) */}
      <div className="mt-4 pt-4 border-t border-gray-200 dark:border-white/5 flex-shrink-0">
        <NavLink to="/settings" className={({ isActive }) => cn("flex items-center gap-2 text-sm font-medium transition p-3 rounded-lg w-full", isActive ? "bg-apple-accent/10 text-apple-accent" : "text-apple-subtext hover:text-apple-text hover:bg-black/5 dark:hover:bg-white/5")}>
          <Settings size={18} /> Settings
        </NavLink>
      </div>
    </div>
  );
}

function App() {
  // Global state for the "Create Playlist" modal
  const [isCreatingPlaylist, setIsCreatingPlaylist] = useState(false);

  // Keyboard shortcuts
  useKeyboardShortcuts({
    onFocusSearch: () => {
      const searchInput = document.querySelector('input[placeholder="Search Library"]') as HTMLInputElement;
      searchInput?.focus();
    }
  });

  return (
    <BrowserRouter>
      <div className="min-h-screen flex flex-col bg-apple-gray transition-colors duration-300 text-apple-text font-sans selection:bg-apple-accent/30">

        {/* Sidebar (Hidden on mobile, visible on lg screens) */}
        <Sidebar setIsCreatingPlaylist={setIsCreatingPlaylist} />

        {/* Main Content Area */}
        <div className="lg:ml-64 flex-1 flex flex-col h-screen pb-24 relative z-0 overflow-hidden">
          <Routes>
            <Route path="/" element={<LibraryPage />} />
            <Route path="/search" element={<SearchPage />} />
            <Route path="/albums" element={<AlbumsPage />} />
            <Route path="/albums/:id" element={<AlbumDetail />} />
            <Route path="/artists" element={<ArtistsPage />} />
            <Route path="/playlists" element={<AllPlaylistsPage />} />
            <Route path="/playlists/:id" element={<PlaylistDetail />} />
            <Route path="/settings" element={<SettingsPage />} />
            <Route path="/artists/:name" element={<ArtistDetail />} />
          </Routes>
        </div>

        {/* Global Fixed Components */}
        <Player />
        <ToastContainer />

        {/* Modals */}
        {isCreatingPlaylist && <CreatePlaylistModal onClose={() => setIsCreatingPlaylist(false)} />}
      </div>
    </BrowserRouter>
  );
}

export default App;