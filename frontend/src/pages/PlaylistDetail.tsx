import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { getPlaylistSongs, getPlaylists, deletePlaylist, getCoverUrl, removeSongFromPlaylist, reorderPlaylist } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { useThemeStore } from '../stores/themeStore';
import { useToastStore } from '../stores/toastStore';
import { formatTime, cn, shuffleArray } from '../lib/utils';
import { Play, Trash2, Music, ListMusic, Shuffle, GripVertical, X } from 'lucide-react';
import { SongListSkeleton } from '../components/Skeletons';
import { useConfirm } from '../stores/confirmStore';
import { Song } from '../types';
import { SongContextMenu } from '../components/ContextMenu';

import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent
} from '@dnd-kit/core';
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
  useSortable
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';

// Wrapper type for DnD stability (handles duplicates)
interface PlaylistItem extends Song {
  uniqueId: string;
}

function SortableRow({
  item,
  index,
  isActive,
  showArt,
  onPlay,
  onRemove
}: {
  item: PlaylistItem;
  index: number;
  isActive: boolean;
  showArt: boolean;
  onPlay: () => void;
  onRemove: (e: React.MouseEvent) => void;
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging
  } = useSortable({ id: item.uniqueId });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    zIndex: isDragging ? 10 : 1,
    opacity: isDragging ? 0.8 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={cn(
        "flex items-center px-4 py-2 hover:bg-gray-200/50 dark:hover:bg-white/5 group transition-colors border-b border-gray-100 dark:border-white/5 h-14 bg-apple-gray",
        isActive && "bg-gray-200 dark:bg-white/10"
      )}
    >
      {/* Drag Handle */}
      <div
        {...attributes}
        {...listeners}
        className="w-8 flex items-center justify-center cursor-grab active:cursor-grabbing text-apple-subtext hover:text-apple-text mr-2 touch-none"
      >
        <GripVertical size={16} />
      </div>

      {/* Index / Play */}
      <div
        className="w-8 text-center text-sm text-apple-subtext relative flex-shrink-0 cursor-pointer"
        onClick={onPlay}
      >
        <span className={cn("group-hover:hidden", isActive && "text-apple-accent font-bold")}>{index + 1}</span>
        <span className="hidden group-hover:flex items-center justify-center absolute inset-0">
          <Play size={12} fill="currentColor" className={isActive ? "text-apple-accent" : "text-apple-text"} />
        </span>
      </div>

      {showArt && (
        <div className="w-12 flex-shrink-0 flex justify-center ml-2 cursor-pointer" onClick={onPlay}>
          <div className="w-9 h-9 rounded overflow-hidden bg-gray-200 dark:bg-white/10 relative border border-black/5">
            {item.album_id ? (
              <img src={getCoverUrl(item.album_id)} className="w-full h-full object-cover" loading="lazy" />
            ) : <Music size={14} className="m-auto text-gray-400" />}
          </div>
        </div>
      )}

      <div className="flex-1 min-w-0 pl-4 pr-4 cursor-pointer" onClick={onPlay}>
        <div className={cn("font-medium truncate text-sm", isActive ? "text-apple-accent" : "text-apple-text")}>
          {item.title}
        </div>
        <div className="md:hidden text-xs text-apple-subtext truncate">{item.artist}</div>
      </div>

      <div className="w-1/3 hidden md:block text-sm text-apple-subtext truncate cursor-pointer" onClick={onPlay}>{item.artist}</div>
      <div className="w-16 text-right text-sm text-apple-subtext tabular-nums mr-4 cursor-pointer" onClick={onPlay}>{formatTime(item.duration)}</div>

      {/* Actions */}
      <div className="flex items-center gap-1">
        <SongContextMenu song={item} onAddToPlaylist={() => { }} />
        <button
          onClick={onRemove}
          className="p-1.5 text-apple-subtext hover:text-red-500 hover:bg-black/5 dark:hover:bg-white/10 rounded transition"
          title="Remove from playlist"
        >
          <X size={16} />
        </button>
      </div>
    </div>
  );
}

export function PlaylistDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { addToast } = useToastStore();

  const { playSong, setQueue, currentSong } = usePlayerStore();
  const { showSongListArt } = useThemeStore();
  const confirm = useConfirm();

  // 1. Get Playlist Metadata
  const { data: playlists } = useQuery({ queryKey: ['playlists'], queryFn: getPlaylists });
  const playlistInfo = playlists?.find(p => p.id === Number(id));

  // 2. Get Songs
  const { data: serverSongs, isLoading } = useQuery({
    queryKey: ['playlist-songs', id],
    queryFn: () => getPlaylistSongs(id!),
    enabled: !!id,
  });

  // Local state for optimistic DnD
  const [items, setItems] = useState<PlaylistItem[]>([]);

  // Sync server data to local state
  // eslint-disable-next-line react-hooks/set-state-in-effect -- Intentional: sync optimistic state with server data
  useEffect(() => {
    if (serverSongs) {
      setItems(serverSongs.map((s, i) => ({ ...s, uniqueId: `${s.id}-${i}` })));
    }
  }, [serverSongs]);

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  );

  const handleDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;

    if (over && active.id !== over.id) {
      setItems((items) => {
        const oldIndex = items.findIndex((i) => i.uniqueId === active.id);
        const newIndex = items.findIndex((i) => i.uniqueId === over.id);
        const newItems = arrayMove(items, oldIndex, newIndex);

        // Optimistic update
        reorderPlaylist(Number(id), newItems.map(i => i.id)).catch(() => {
          addToast('Failed to save order', 'error');
          queryClient.invalidateQueries({ queryKey: ['playlist-songs', id] });
        });

        return newItems;
      });
    }
  };

  const handleRemoveSong = async (e: React.MouseEvent, songId: number, itemUniqueId: string) => {
    e.stopPropagation();

    // Optimistic removal
    setItems(prev => prev.filter(i => i.uniqueId !== itemUniqueId));

    try {
      await removeSongFromPlaylist(Number(id), songId);
      // addToast('Song removed'); // Optional toast, maybe too noisy
    } catch {
      addToast('Failed to remove song', 'error');
      queryClient.invalidateQueries({ queryKey: ['playlist-songs', id] });
    }
  };

  const handlePlay = (song: PlaylistItem) => {
    // Strip uniqueId for player - using rest spread to omit it
    const { uniqueId: _, ...rawSong } = song;
    // Set queue to current visible order (omitting uniqueId from each)
    const queue = items.map(({ uniqueId: __, ...s }) => s);
    setQueue(queue);
    playSong(rawSong);
  };

  const handlePlayAll = () => {
    if (items.length > 0) {
      const queue = items.map(({ uniqueId: _, ...s }) => s);
      setQueue(queue);
      usePlayerStore.setState({ isShuffle: false });
      playSong(queue[0]);
    }
  };

  const handleShuffleAll = () => {
    if (items.length > 0) {
      const queue = items.map(({ uniqueId: _, ...s }) => s);
      const shuffled = shuffleArray(queue);
      usePlayerStore.setState({ isShuffle: true, queue: shuffled, originalQueue: queue });
      playSong(shuffled[0]);
    }
  };

  const handleDeletePlaylist = async () => {
    const confirmed = await confirm({
      title: 'Delete Playlist',
      message: `Are you sure you want to delete "${playlistInfo?.name}"?`,
      confirmText: 'Delete',
      variant: 'danger',
    });

    if (confirmed) {
      await deletePlaylist(Number(id));
      queryClient.invalidateQueries({ queryKey: ['playlists'] });
      addToast('Playlist deleted');
      navigate('/');
    }
  };

  if (!playlistInfo) return <div className="p-8 text-apple-subtext">Playlist not found</div>;

  return (
    <div className="h-full flex flex-col bg-apple-gray overflow-hidden">
      {/* Header Section */}
      <div className="p-8 pb-6 flex items-end gap-6 border-b border-gray-200 dark:border-white/10 flex-shrink-0 bg-apple-gray z-10">
        <div className="w-32 h-32 sm:w-40 sm:h-40 bg-gradient-to-br from-pink-500 to-orange-400 rounded-lg shadow-xl flex items-center justify-center text-white">
          {items.length > 0 && items[0].album_id ? (
            <img src={getCoverUrl(items[0].album_id)} className="w-full h-full object-cover rounded-lg" />
          ) : (
            <ListMusic size={48} />
          )}
        </div>
        <div className="flex-1 mb-1">
          <h5 className="text-xs font-bold text-apple-accent uppercase tracking-widest mb-1">Playlist</h5>
          <h1 className="text-3xl sm:text-5xl font-bold text-apple-text mb-3">{playlistInfo.name}</h1>
          <div className="flex items-center gap-3 text-apple-subtext text-sm font-medium">
            <span>{items.length} Songs</span>
            {items.length > 0 && (
              <div className="flex gap-2">
                <button onClick={handlePlayAll} className="px-4 py-1.5 bg-apple-accent text-white rounded-full font-semibold flex items-center gap-2 hover:opacity-90 transition text-sm shadow-lg">
                  <Play size={14} fill="currentColor" /> Play
                </button>
                <button onClick={handleShuffleAll} className="px-4 py-1.5 bg-gray-200 dark:bg-white/10 text-apple-text rounded-full font-semibold flex items-center gap-2 hover:bg-gray-300 dark:hover:bg-white/20 transition text-sm">
                  <Shuffle size={14} /> Shuffle
                </button>
              </div>
            )}
          </div>
        </div>
        <button onClick={handleDeletePlaylist} className="p-2 text-apple-subtext hover:text-red-500 transition mb-2 bg-gray-100 dark:bg-white/5 rounded-lg" title="Delete Playlist">
          <Trash2 size={20} />
        </button>
      </div>

      {/* Song List Table */}
      <div className="flex-1 overflow-y-auto pb-32">
        {isLoading ? (
          <SongListSkeleton count={8} />
        ) : items.length === 0 ? (
          <div className="p-12 text-center text-apple-subtext">This playlist is empty. Add songs from the Library.</div>
        ) : (
          <div className="w-full">
            <div className="flex items-center px-4 py-2 text-xs font-semibold text-apple-subtext uppercase tracking-wider sticky top-0 bg-apple-gray/95 backdrop-blur z-20 border-b border-gray-200 dark:border-white/5">
              <div className="w-8 ml-2"></div>{/* Grip placeholder */}
              <div className="w-8 text-center">#</div>
              {showSongListArt && <div className="w-12 ml-2"></div>}
              <div className="flex-1 pl-4">Title</div>
              <div className="w-1/3 hidden md:block">Artist</div>
              <div className="w-16 text-right mr-4">Time</div>
              <div className="w-6"></div>{/* Delete placeholder */}
            </div>

            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragEnd={handleDragEnd}
            >
              <SortableContext
                items={items.map(i => i.uniqueId)}
                strategy={verticalListSortingStrategy}
              >
                {items.map((song, index) => (
                  <SortableRow
                    key={song.uniqueId}
                    item={song}
                    index={index}
                    isActive={currentSong?.id === song.id}
                    showArt={showSongListArt}
                    onPlay={() => handlePlay(song)}
                    onRemove={(e) => handleRemoveSong(e, song.id, song.uniqueId)}
                  />
                ))}
              </SortableContext>
            </DndContext>
          </div>
        )}
      </div>
    </div>
  );
}