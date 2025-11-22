import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Virtuoso } from 'react-virtuoso';
import { getSongs, getCoverUrl } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { useThemeStore } from '../stores/themeStore';
import { formatTime, cn } from '../lib/utils';
import { Play, Music, Shuffle, MoreHorizontal, Check, ArrowUp, ArrowDown, MoreVertical } from 'lucide-react';
import { AddToPlaylistModal } from './AddToPlaylistModal';

export function SongList() {
  const [sortBy, setSortBy] = useState('title');
  const [order, setOrder] = useState<'asc' | 'desc'>('asc');
  const [showMenu, setShowMenu] = useState(false);

  const { data: songs, isLoading } = useQuery({
    queryKey: ['songs', sortBy, order],
    queryFn: () => getSongs(0, 5000, sortBy, order),
  });

  const { playSong, currentSong, setQueue } = usePlayerStore();
  const { showSongListArt, setShowSongListArt } = useThemeStore();
  const [playlistModalSong, setPlaylistModalSong] = useState<number | null>(null);

  const handlePlay = (song: any) => {
    if (songs) setQueue(songs);
    playSong(song);
  };

  const handleInstantPlay = () => {
    if (!songs || songs.length === 0) return;
    setQueue(songs);
    usePlayerStore.setState({ isShuffle: false }); 
    playSong(songs[0]);
  };

  const handleInstantShuffle = () => {
    if (!songs || songs.length === 0) return;
    const newQueue = [...songs].sort(() => Math.random() - 0.5);
    usePlayerStore.setState({ isShuffle: true, queue: newQueue, originalQueue: songs });
    playSong(newQueue[0]);
  };

  const handleSort = (field: string) => {
    if (sortBy === field) setOrder(order === 'asc' ? 'desc' : 'asc');
    else { setSortBy(field); setOrder('asc'); }
  };

  const SortOption = ({ label, field }: { label: string, field: string }) => (
    <button 
        onClick={() => handleSort(field)} 
        className={cn("w-full text-left px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-white/5 flex justify-between items-center transition", sortBy === field && "text-apple-accent font-medium")}
    >
        {label}
        {sortBy === field && (order === 'asc' ? <ArrowUp size={14} /> : <ArrowDown size={14} />)}
    </button>
  );

  if (isLoading) return <div className="p-12 text-center text-apple-subtext">Loading Library...</div>;

  return (
    <div className="w-full bg-apple-gray">
      
      {/* --- CONTROLS & HEADER --- */}
      <div className="bg-apple-gray border-b border-gray-200 dark:border-white/10">
          
          {/* Top Row: Play Buttons & Menu */}
          <div className="px-6 py-4 flex justify-between items-center">
             <div className="flex gap-3">
                <button onClick={handleInstantPlay} className="px-5 py-2 bg-gray-200 dark:bg-white/10 hover:bg-gray-300 dark:hover:bg-white/20 text-apple-text rounded-lg font-semibold flex items-center gap-2 transition shadow-sm text-sm">
                    <Play fill="currentColor" size={16} /> Play
                </button>
                <button onClick={handleInstantShuffle} className="px-5 py-2 bg-gray-200 dark:bg-white/10 hover:bg-gray-300 dark:hover:bg-white/20 text-apple-text rounded-lg font-semibold flex items-center gap-2 transition shadow-sm text-sm">
                    <Shuffle size={16} /> Shuffle
                </button>
             </div>

             <div className="relative">
                <button onClick={() => setShowMenu(!showMenu)} className={cn("p-2 rounded-full transition", showMenu ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "hover:bg-gray-200 dark:hover:bg-white/10 text-apple-subtext")}>
                    <MoreVertical size={20} />
                </button>

                {showMenu && (
                    <div className="absolute right-0 top-10 w-56 bg-white dark:bg-[#2c2c2e] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 py-2 z-50 animate-in fade-in zoom-in-95 duration-100">
                        <div className="px-4 py-2 text-xs font-bold text-apple-subtext uppercase tracking-wider">View</div>
                        <button onClick={() => setShowSongListArt(!showSongListArt)} className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-white/5 flex items-center justify-between text-apple-text transition">
                            <span>Show Artwork</span>
                            {showSongListArt && <Check size={14} className="text-apple-accent" />}
                        </button>
                        <div className="h-px bg-gray-100 dark:bg-white/5 my-2" />
                        <div className="px-4 py-2 text-xs font-bold text-apple-subtext uppercase tracking-wider">Sort By</div>
                        <SortOption label="Title" field="title" />
                        <SortOption label="Artist" field="artist" />
                        <SortOption label="Album" field="album" />
                        <SortOption label="Duration" field="duration" />
                        <SortOption label="Date Added" field="id" />
                    </div>
                )}
             </div>
          </div>

          {/* Bottom Row: Column Labels */}
          <div className="flex items-center px-6 py-2 text-xs font-semibold text-apple-subtext uppercase tracking-wider bg-gray-50/50 dark:bg-white/5">
                <div className="w-10 text-center">#</div>
                {showSongListArt && <div className="w-12"></div>}
                <div className="flex-1 pl-4 cursor-pointer hover:text-apple-text transition" onClick={() => handleSort('title')}>Title</div>
                <div className="w-1/4 hidden md:block cursor-pointer hover:text-apple-text transition" onClick={() => handleSort('artist')}>Artist</div>
                <div className="w-1/4 hidden lg:block cursor-pointer hover:text-apple-text transition pl-4" onClick={() => handleSort('album')}>Album</div>
                <div className="w-16 text-right cursor-pointer hover:text-apple-text transition" onClick={() => handleSort('duration')}>Time</div>
                <div className="w-10"></div>
          </div>
      </div>

      {/* --- SCROLLABLE LIST (Calculated Height) --- */}
      <div style={{ height: 'calc(100vh - 280px)' }}>
        {!songs || songs.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-apple-subtext gap-2">
                <Music size={48} className="opacity-20 mb-2" />
                <p>No songs found.</p>
                <p className="text-xs">Try resetting the library in Settings.</p>
            </div>
        ) : (
            <Virtuoso
                style={{ height: '100%' }}
                data={songs}
                itemContent={(index, song) => {
                const isActive = currentSong?.id === song.id;
                return (
                    <div 
                    className={cn(
                        "flex items-center px-6 py-1.5 hover:bg-gray-200/50 dark:hover:bg-white/5 cursor-pointer group/row transition-colors border-b border-gray-100 dark:border-white/5 h-14",
                        isActive && "bg-gray-200 dark:bg-white/10"
                    )}
                    onClick={() => handlePlay(song)}
                    >
                        <div className="w-10 text-center text-sm text-apple-subtext relative flex-shrink-0">
                            <span className={cn("group-hover/row:hidden", isActive && "text-apple-accent font-bold")}>{index + 1}</span>
                            <span className="hidden group-hover/row:flex items-center justify-center absolute inset-0">
                                <Play size={14} fill="currentColor" className={isActive ? "text-apple-accent" : "text-apple-text"} />
                            </span>
                        </div>

                        {showSongListArt && (
                            <div className="w-12 flex-shrink-0 flex justify-center">
                            <div className="w-9 h-9 rounded bg-gray-200 dark:bg-white/10 relative overflow-hidden border border-black/5 dark:border-white/5">
                                {song.album_id ? (
                                    <img src={getCoverUrl(song.album_id)} className="w-full h-full object-cover" loading="lazy" />
                                ) : <Music size={14} className="m-auto text-gray-400" />}
                            </div>
                            </div>
                        )}

                        <div className="flex-1 min-w-0 pl-4 pr-4">
                            <div className={cn("font-medium truncate text-sm", isActive ? "text-apple-accent" : "text-apple-text")}>
                            {song.title}
                            </div>
                            <div className="md:hidden text-[10px] text-apple-subtext truncate">{song.artist}</div>
                        </div>

                        <div className="w-1/4 hidden md:block text-sm text-apple-subtext truncate">
                            {song.artist}
                        </div>

                        <div className="w-1/4 hidden lg:block text-sm text-apple-subtext truncate pl-4">
                            {song.album?.title}
                        </div>

                        <div className="w-16 text-right text-sm text-apple-subtext tabular-nums">{formatTime(song.duration)}</div>
                        
                        <div className="w-10 flex items-center justify-center">
                            <button 
                                className="p-1.5 text-apple-subtext opacity-0 group-hover/row:opacity-100 transition rounded hover:bg-gray-300 dark:hover:bg-white/10"
                                onClick={(e) => { e.stopPropagation(); setPlaylistModalSong(song.id); }}
                            >
                                <MoreHorizontal size={16} />
                            </button>
                        </div>
                    </div>
                );
                }}
            />
        )}
      </div>

      {playlistModalSong && <AddToPlaylistModal songIds={[playlistModalSong]} onClose={() => setPlaylistModalSong(null)} />}
    </div>
  );
}