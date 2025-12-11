// Queue management panel with drag-and-drop
import { useState } from 'react';
import { X, GripVertical, Trash2, ListMusic, Save } from 'lucide-react';
import { usePlayerStore } from '../../stores/playerStore';
import { getCoverUrl, createPlaylist, addToPlaylist } from '../../lib/api';
import { formatTime, cn } from '../../lib/utils';
import { useQueryClient } from '@tanstack/react-query';
import {
    DndContext,
    closestCenter,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
    DragEndEvent,
} from '@dnd-kit/core';
import {
    SortableContext,
    sortableKeyboardCoordinates,
    useSortable,
    verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { Song } from '../../types';

interface QueuePanelProps {
    isOpen: boolean;
    onClose: () => void;
}

interface SortableSongItemProps {
    song: Song;
    isCurrent: boolean;
    onRemove: () => void;
}

function SortableSongItem({ song, isCurrent, onRemove }: SortableSongItemProps) {
    const { playSong, queue } = usePlayerStore();
    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
        isDragging,
    } = useSortable({ id: song.id });

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
    };

    return (
        <div
            ref={setNodeRef}
            style={style}
            className={cn(
                "flex items-center gap-3 p-2 rounded-lg group transition-colors",
                isCurrent ? "bg-apple-accent/10" : "hover:bg-gray-100 dark:hover:bg-white/5",
                isDragging && "opacity-50"
            )}
        >
            <button
                {...attributes}
                {...listeners}
                className="cursor-grab active:cursor-grabbing text-apple-subtext hover:text-apple-text transition p-1"
            >
                <GripVertical size={16} />
            </button>

            <img
                src={getCoverUrl(song.album_id ?? 0)}
                alt={song.title}
                className="w-10 h-10 rounded object-cover"
            />

            <div
                className="flex-1 min-w-0 cursor-pointer"
                onClick={() => {
                    usePlayerStore.getState().setQueue(queue);
                    playSong(song);
                }}
            >
                <div className={cn(
                    "text-sm font-medium truncate",
                    isCurrent ? "text-apple-accent" : "text-apple-text"
                )}>
                    {song.title}
                </div>
                <div className="text-xs text-apple-subtext truncate">
                    {song.artist}
                </div>
            </div>

            <div className="text-xs text-apple-subtext tabular-nums">
                {formatTime(song.duration)}
            </div>

            {!isCurrent && (
                <button
                    onClick={onRemove}
                    className="opacity-0 group-hover:opacity-100 p-1 text-apple-subtext hover:text-red-500 transition"
                >
                    <Trash2 size={14} />
                </button>
            )}
        </div>
    );
}

export function QueuePanel({ isOpen, onClose }: QueuePanelProps) {
    const queryClient = useQueryClient();
    const { queue, currentSong, currentIndex, reorderQueue, removeFromQueue, clearQueue } = usePlayerStore();
    const [isSaving, setIsSaving] = useState(false);

    const sensors = useSensors(
        useSensor(PointerSensor),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates,
        })
    );

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;

        if (over && active.id !== over.id) {
            const oldIndex = queue.findIndex((song) => song.id === active.id);
            const newIndex = queue.findIndex((song) => song.id === over.id);
            reorderQueue(oldIndex, newIndex);
        }
    };

    const upcomingSongs = queue.slice(currentIndex + 1);

    const handleSaveAsPlaylist = async () => {
        if (queue.length === 0) return;
        setIsSaving(true);
        try {
            const name = `Queue - ${new Date().toLocaleDateString()}`;
            const playlist = await createPlaylist(name);
            await addToPlaylist(playlist.id, queue.map(s => s.id));
            queryClient.invalidateQueries({ queryKey: ['playlists'] });
            alert(`Saved as "${name}"`);
        } catch (error) {
            console.error('Failed to save queue as playlist:', error);
        } finally {
            setIsSaving(false);
        }
    };

    return (
        <>
            {/* Backdrop */}
            <div
                className={cn(
                    "fixed inset-0 bg-black/50 backdrop-blur-sm transition-opacity z-40",
                    isOpen ? "opacity-100" : "opacity-0 pointer-events-none"
                )}
                onClick={onClose}
            />

            {/* Panel */}
            <div
                className={cn(
                    "fixed top-0 right-0 h-full w-96 bg-white dark:bg-gray-900 shadow-2xl transform transition-transform duration-300 ease-out z-50 flex flex-col",
                    isOpen ? "translate-x-0" : "translate-x-full"
                )}
            >
                {/* Header */}
                <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-white/10">
                    <div className="flex items-center gap-2">
                        <ListMusic size={20} className="text-apple-accent" />
                        <h2 className="text-lg font-semibold text-apple-text">
                            Queue ({queue.length})
                        </h2>
                    </div>
                    <div className="flex items-center gap-2">
                        {queue.length > 0 && (
                            <>
                                <button
                                    onClick={handleSaveAsPlaylist}
                                    disabled={isSaving}
                                    className="text-xs text-apple-subtext hover:text-apple-accent transition px-2 py-1 rounded hover:bg-gray-100 dark:hover:bg-white/5 flex items-center gap-1 disabled:opacity-50"
                                    title="Save as Playlist"
                                >
                                    <Save size={12} /> Save
                                </button>
                                <button
                                    onClick={clearQueue}
                                    className="text-xs text-apple-subtext hover:text-red-500 transition px-2 py-1 rounded hover:bg-gray-100 dark:hover:bg-white/5"
                                >
                                    Clear All
                                </button>
                            </>
                        )}
                        <button
                            onClick={onClose}
                            className="p-2 hover:bg-gray-100 dark:hover:bg-white/10 rounded-full transition"
                        >
                            <X size={20} className="text-apple-subtext" />
                        </button>
                    </div>
                </div>

                {/* Content */}
                <div className="flex-1 overflow-y-auto p-4">
                    {queue.length === 0 ? (
                        <div className="flex flex-col items-center justify-center h-full text-center">
                            <ListMusic size={48} className="text-apple-subtext/30 mb-4" />
                            <p className="text-apple-subtext">Queue is empty</p>
                            <p className="text-xs text-apple-subtext/70 mt-1">
                                Play a song to start your queue
                            </p>
                        </div>
                    ) : (
                        <div className="space-y-6">
                            {/* Now Playing */}
                            {currentSong && (
                                <div>
                                    <h3 className="text-xs font-semibold text-apple-subtext uppercase tracking-wider mb-2">
                                        Now Playing
                                    </h3>
                                    <div className="flex items-center gap-3 p-3 rounded-lg bg-apple-accent/10 border border-apple-accent/20">
                                        <img
                                            src={getCoverUrl(currentSong.album_id ?? 0)}
                                            alt={currentSong.title}
                                            className="w-12 h-12 rounded object-cover"
                                        />
                                        <div className="flex-1 min-w-0">
                                            <div className="text-sm font-semibold text-apple-accent truncate">
                                                {currentSong.title}
                                            </div>
                                            <div className="text-xs text-apple-subtext truncate">
                                                {currentSong.artist}
                                            </div>
                                        </div>
                                        <div className="text-xs text-apple-subtext tabular-nums">
                                            {formatTime(currentSong.duration)}
                                        </div>
                                    </div>
                                </div>
                            )}

                            {/* Next Up (Lazy Render Limit for Performance) */}
                            {upcomingSongs.length > 0 && (
                                <div>
                                    <h3 className="text-xs font-semibold text-apple-subtext uppercase tracking-wider mb-2">
                                        Next Up ({upcomingSongs.length})
                                    </h3>
                                    <DndContext
                                        sensors={sensors}
                                        collisionDetection={closestCenter}
                                        onDragEnd={handleDragEnd}
                                    >
                                        <SortableContext
                                            items={upcomingSongs.slice(0, 50).map(s => s.id)}
                                            strategy={verticalListSortingStrategy}
                                        >
                                            <div className="space-y-1">
                                                {upcomingSongs.slice(0, 50).map((song, idx) => (
                                                    <SortableSongItem
                                                        key={song.id}
                                                        song={song}
                                                        isCurrent={false}
                                                        onRemove={() => removeFromQueue(currentIndex + 1 + idx)}
                                                    />
                                                ))}
                                            </div>
                                        </SortableContext>
                                    </DndContext>

                                    {upcomingSongs.length > 50 && (
                                        <div className="py-4 text-center text-xs text-apple-subtext">
                                            + {upcomingSongs.length - 50} more songs in queue...
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>
                    )}
                </div>
            </div>
        </>
    );
}
