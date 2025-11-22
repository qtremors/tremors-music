import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { getPlaylists, deletePlaylist, renamePlaylist, getPlaylistSongs, getCoverUrl } from '../lib/api';
import { Plus, MoreVertical, Trash2, Edit2, ListMusic, Play } from 'lucide-react';
import { usePlayerStore } from '../stores/playerStore';
import { useToastStore } from '../stores/toastStore';
import { CreatePlaylistModal } from '../components/CreatePlaylistModal';

// --- Sub-Component for Individual Playlist Card ---
function PlaylistCard({ playlist }: { playlist: any }) {
  const navigate = useNavigate();
  const { playSong, setQueue } = usePlayerStore();
  const { addToast } = useToastStore();
  const queryClient = useQueryClient();
  const [showMenu, setShowMenu] = useState(false);

  // Fetch first song to get cover art
  const { data: songs } = useQuery({
    queryKey: ['playlist-songs', playlist.id],
    queryFn: () => getPlaylistSongs(playlist.id.toString()),
  });

  const handlePlay = (e: any) => {
    e.stopPropagation();
    if (songs && songs.length > 0) {
      setQueue(songs);
      playSong(songs[0]);
    } else {
        addToast('Playlist is empty', 'error');
    }
  };

  const handleDelete = async (e: any) => {
    e.stopPropagation();
    if (confirm(`Delete "${playlist.name}"?`)) {
      await deletePlaylist(playlist.id);
      queryClient.invalidateQueries({ queryKey: ['playlists'] });
      addToast('Playlist deleted');
    }
  };

  const handleRename = async (e: any) => {
    e.stopPropagation();
    const newName = prompt("Rename Playlist:", playlist.name);
    if (newName) {
      await renamePlaylist(playlist.id, newName);
      queryClient.invalidateQueries({ queryKey: ['playlists'] });
    }
    setShowMenu(false);
  };

  // Determine Cover Art (First song's album)
  const coverArtId = songs?.[0]?.album_id;

  return (
    <div 
      className="group cursor-pointer relative"
      onClick={() => navigate(`/playlists/${playlist.id}`)}
      onMouseLeave={() => setShowMenu(false)}
    >
      {/* Art Container */}
      <div className="aspect-square bg-gray-200 dark:bg-white/5 rounded-lg shadow-sm overflow-hidden relative border border-black/5 dark:border-white/5 group-hover:shadow-md transition-all">
        {coverArtId ? (
          <img 
            src={getCoverUrl(coverArtId)} 
            alt={playlist.name}
            className="w-full h-full object-cover transition duration-500 group-hover:scale-105"
            loading="lazy"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-apple-subtext bg-gray-100 dark:bg-white/5">
            <ListMusic size={48} className="opacity-50" />
          </div>
        )}

        {/* Play Overlay */}
        <div className="absolute inset-0 bg-black/20 opacity-0 group-hover:opacity-100 transition flex items-center justify-center">
           <button 
             onClick={handlePlay}
             className="w-12 h-12 bg-apple-accent text-white rounded-full flex items-center justify-center shadow-lg transform scale-90 group-hover:scale-100 transition hover:bg-apple-accent/90"
           >
             <Play size={24} fill="currentColor" className="ml-1" />
           </button>
        </div>

        {/* Menu Button (Top Right) */}
        <button 
          onClick={(e) => { e.stopPropagation(); setShowMenu(!showMenu); }}
          className={`absolute top-2 right-2 p-1.5 rounded-full bg-black/50 text-white backdrop-blur-md transition ${showMenu ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}`}
        >
          <MoreVertical size={16} />
        </button>

        {/* Dropdown Menu */}
        {showMenu && (
          <div className="absolute top-10 right-2 w-32 bg-white dark:bg-[#2c2c2e] rounded-lg shadow-xl border border-gray-200 dark:border-white/10 py-1 z-20 animate-in fade-in zoom-in-95 duration-100">
            <button onClick={handleRename} className="w-full text-left px-3 py-2 text-sm text-apple-text hover:bg-gray-100 dark:hover:bg-white/10 flex items-center gap-2">
              <Edit2 size={14} /> Rename
            </button>
            <button onClick={handleDelete} className="w-full text-left px-3 py-2 text-sm text-red-500 hover:bg-gray-100 dark:hover:bg-white/10 flex items-center gap-2">
              <Trash2 size={14} /> Delete
            </button>
          </div>
        )}
      </div>

      {/* Text Info */}
      <div className="mt-3">
        <h3 className="font-medium text-apple-text truncate">{playlist.name}</h3>
        <p className="text-xs text-apple-subtext">{songs?.length || 0} Songs</p>
      </div>
    </div>
  );
}

// --- Main Page Component ---
export function AllPlaylistsPage() {
  const { data: playlists, isLoading } = useQuery({
    queryKey: ['playlists'],
    queryFn: getPlaylists,
  });
  
  const [isCreating, setIsCreating] = useState(false); 

  if (isLoading) return <div className="p-12 text-center text-apple-subtext">Loading Playlists...</div>;

  return (
    <div className="h-full flex flex-col bg-apple-gray">
      <header className="h-16 flex items-center px-8 border-b border-gray-200 dark:border-white/10 bg-apple-gray/95 backdrop-blur z-10">
        <h2 className="text-xl font-semibold text-apple-text">All Playlists</h2>
      </header>

      <main className="flex-1 overflow-y-auto p-8 pb-32">
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
          
          {/* 1. Existing Playlists */}
          {playlists?.map((playlist) => (
            <PlaylistCard key={playlist.id} playlist={playlist} />
          ))}

          {/* 2. The "Create New" Card */}
          <button 
            onClick={() => setIsCreating(true)}
            className="group aspect-square rounded-lg border-2 border-dashed border-gray-300 dark:border-white/10 flex flex-col items-center justify-center hover:border-apple-accent hover:bg-apple-accent/5 transition cursor-pointer"
          >
            <div className="w-16 h-16 rounded-full bg-gray-200 dark:bg-white/5 flex items-center justify-center text-apple-subtext group-hover:text-apple-accent group-hover:bg-apple-accent/10 transition">
               <Plus size={32} />
            </div>
            <span className="mt-4 font-medium text-apple-subtext group-hover:text-apple-accent">Create New</span>
          </button>

        </div>
      </main>

      {/* Render Modal */}
      {isCreating && <CreatePlaylistModal onClose={() => setIsCreating(false)} />}
    </div>
  );
}