import { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { getPlaylists, createPlaylist, addToPlaylist } from '../lib/api';
import { useToastStore } from '../stores/toastStore';
import { X, Plus, Check, ListMusic } from 'lucide-react';

interface Props {
  songIds: number[];
  onClose: () => void;
}

export function AddToPlaylistModal({ songIds, onClose }: Props) {
  const queryClient = useQueryClient();
  const { addToast } = useToastStore();
  const [newPlaylistName, setNewPlaylistName] = useState('');
  const [isCreating, setIsCreating] = useState(false);

  const { data: playlists } = useQuery({ queryKey: ['playlists'], queryFn: getPlaylists });

  const handleCreate = async () => {
    if (!newPlaylistName) return;
    try {
      const pl = await createPlaylist(newPlaylistName);
      await addToPlaylist(pl.id, songIds);
      queryClient.invalidateQueries({ queryKey: ['playlists'] });
      addToast(`Created "${pl.name}" and added songs`); // <--- Toast
      onClose();
    } catch {
      addToast('Failed to create playlist', 'error');
    }
  };

  const handleAdd = async (playlistId: number) => {
    try {
      await addToPlaylist(playlistId, songIds);
      addToast('Songs added to playlist'); // <--- Toast
      onClose();
    } catch {
      addToast('Failed to add songs', 'error');
    }
  };

  return (
    <div className="fixed inset-0 z-[200] bg-black/20 flex items-center justify-center backdrop-blur-sm">
      <div className="bg-white dark:bg-[#1c1c1e] p-6 rounded-2xl w-full max-w-sm shadow-2xl border border-white/10 animate-in zoom-in-95 duration-200">
        <div className="flex justify-between items-center mb-4">
          <h3 className="font-bold text-apple-text">Add to Playlist</h3>
          <button onClick={onClose}><X size={20} className="text-apple-subtext hover:text-apple-text" /></button>
        </div>

        <div className="max-h-60 overflow-y-auto space-y-1 mb-4 no-scrollbar">
          {playlists?.map(pl => (
            <button
              key={pl.id}
              onClick={() => handleAdd(pl.id)}
              className="w-full text-left p-3 rounded-lg hover:bg-gray-100 dark:hover:bg-white/10 text-apple-text flex items-center gap-3 transition"
            >
              <span className="w-8 h-8 bg-gray-100 dark:bg-white/10 rounded-md flex items-center justify-center text-apple-accent">
                <ListMusic size={16} />
              </span>
              <span className="truncate font-medium text-sm">{pl.name}</span>
            </button>
          ))}
          {playlists?.length === 0 && <p className="text-center text-apple-subtext py-4 text-sm">No playlists yet.</p>}
        </div>

        <div className="pt-4 border-t border-gray-200 dark:border-white/10">
          {!isCreating ? (
            <button
              onClick={() => setIsCreating(true)}
              className="w-full py-2.5 bg-apple-accent/10 text-apple-accent rounded-lg font-medium hover:bg-apple-accent/20 transition flex items-center justify-center gap-2 text-sm"
            >
              <Plus size={16} /> New Playlist
            </button>
          ) : (
            <div className="flex gap-2">
              <input
                autoFocus
                className="flex-1 bg-gray-100 dark:bg-white/10 rounded-lg px-3 py-2 outline-none text-apple-text text-sm focus:ring-1 focus:ring-apple-accent"
                placeholder="Playlist Name..."
                value={newPlaylistName}
                onChange={e => setNewPlaylistName(e.target.value)}
                onKeyDown={e => e.key === 'Enter' && handleCreate()}
              />
              <button onClick={handleCreate} className="p-2 bg-apple-accent text-white rounded-lg hover:opacity-90">
                <Check size={18} />
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}