import { useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { createPlaylist } from '../lib/api';
import { useToastStore } from '../stores/toastStore';
import { X, ListMusic } from 'lucide-react';

interface Props {
  onClose: () => void;
}

export function CreatePlaylistModal({ onClose }: Props) {
  const [name, setName] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const queryClient = useQueryClient();
  const { addToast } = useToastStore();

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault();
    if (!name.trim()) return;

    setIsLoading(true);
    try {
      await createPlaylist(name);
      queryClient.invalidateQueries({ queryKey: ['playlists'] });
      addToast(`Created playlist "${name}"`);
      onClose();
    } catch {
      addToast('Failed to create playlist', 'error');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-[200] bg-black/20 backdrop-blur-sm flex items-center justify-center">
      <div className="bg-white dark:bg-[#1c1c1e] w-full max-w-sm p-6 rounded-2xl shadow-2xl border border-white/10 animate-in zoom-in-95 duration-200">

        {/* Header */}
        <div className="flex justify-between items-center mb-6">
          <h3 className="font-bold text-lg text-apple-text">New Playlist</h3>
          <button onClick={onClose} className="text-apple-subtext hover:text-apple-text transition">
            <X size={20} />
          </button>
        </div>

        {/* Icon Preview */}
        <div className="flex justify-center mb-6">
          <div className="w-24 h-24 bg-gray-100 dark:bg-white/5 rounded-xl flex items-center justify-center shadow-inner">
            <ListMusic size={40} className="text-apple-accent opacity-80" />
          </div>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <input
              autoFocus
              type="text"
              placeholder="Playlist Name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full bg-gray-100 dark:bg-white/5 border border-transparent focus:border-apple-accent/50 rounded-lg px-4 py-3 text-apple-text outline-none transition-all placeholder:text-apple-subtext"
            />
          </div>

          <div className="flex gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 py-3 rounded-lg font-medium text-apple-text hover:bg-gray-100 dark:hover:bg-white/5 transition"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!name.trim() || isLoading}
              className="flex-1 py-3 bg-apple-accent text-white rounded-lg font-medium hover:opacity-90 transition disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {isLoading ? 'Creating...' : 'Create'}
            </button>
          </div>
        </form>

      </div>
    </div>
  );
}