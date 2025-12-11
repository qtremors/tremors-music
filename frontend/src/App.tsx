import { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route, useLocation } from 'react-router-dom';
import { AnimatePresence } from 'framer-motion';
import { ErrorBoundary } from './components/ErrorBoundary';
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
import { GenresPage } from './pages/GenresPage';
import { GenreDetail } from './pages/GenreDetail';
import { ToastContainer } from './components/ToastContainer';
import { ConfirmDialog } from './components/ConfirmDialog';
import { CreatePlaylistModal } from './components/CreatePlaylistModal';
import { usePlayerStore } from './stores/playerStore';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { SmartPlaylistDetail } from './pages/SmartPlaylistDetail';
import { Sidebar } from './components/Sidebar';

function AppContent() {
  const location = useLocation();
  const [isCreatingPlaylist, setIsCreatingPlaylist] = useState(false);

  useKeyboardShortcuts({
    onFocusSearch: () => {
      const searchInput = document.querySelector('input[placeholder="Search Library"]') as HTMLInputElement;
      searchInput?.focus();
    }
  });

  // Validate player state on mount (handle stale localStorage data)
  useEffect(() => {
    usePlayerStore.getState().validateState();
  }, []);

  return (
    <div className="min-h-screen flex flex-col bg-apple-gray transition-colors duration-300 text-apple-text font-sans selection:bg-apple-accent/30">
      <Sidebar setIsCreatingPlaylist={setIsCreatingPlaylist} />

      <div className="lg:ml-64 flex-1 flex flex-col h-screen pb-24 relative z-0 overflow-hidden">
        <AnimatePresence mode="wait">
          <Routes location={location} key={location.pathname}>
            <Route path="/" element={<LibraryPage />} />
            <Route path="/search" element={<SearchPage />} />
            <Route path="/albums" element={<AlbumsPage />} />
            <Route path="/albums/:id" element={<AlbumDetail />} />
            <Route path="/artists" element={<ArtistsPage />} />
            <Route path="/genres" element={<GenresPage />} />
            <Route path="/genres/:name" element={<GenreDetail />} />
            <Route path="/playlists" element={<AllPlaylistsPage />} />
            <Route path="/playlists/:id" element={<PlaylistDetail />} />
            <Route path="/settings" element={<SettingsPage />} />
            <Route path="/artists/:name" element={<ArtistDetail />} />
            <Route path="/smart/:type" element={<SmartPlaylistDetail />} />
          </Routes>
        </AnimatePresence>
      </div>

      <Player />
      <ToastContainer />
      <ConfirmDialog />
      {isCreatingPlaylist && <CreatePlaylistModal onClose={() => setIsCreatingPlaylist(false)} />}
    </div>
  );
}

export default function App() {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <AppContent />
      </BrowserRouter>
    </ErrorBoundary>
  );
}