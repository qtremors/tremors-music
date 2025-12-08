import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { getFavorites, getRecentlyAdded, getMostPlayed, getCoverUrl, toggleFavorite } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { formatTime, cn } from '../lib/utils';
import { Play, Shuffle, ArrowLeft, Heart, Clock, TrendingUp, Sparkles } from 'lucide-react';
import { SongListSkeleton } from '../components/Skeletons';
import { SongContextMenu, ArtistLink } from '../components/ContextMenu';
import { AddToPlaylistModal } from '../components/AddToPlaylistModal';
import { Song } from '../types';

type SmartPlaylistType = 'favorites' | 'recently-added' | 'most-played';

const playlistConfig = {
    'favorites': {
        title: 'Favorites',
        icon: Heart,
        gradient: 'from-pink-500 to-rose-500',
        description: 'Your most loved songs',
        fetcher: getFavorites,
    },
    'recently-added': {
        title: 'Recently Added',
        icon: Sparkles,
        gradient: 'from-green-500 to-emerald-500',
        description: 'Fresh additions to your library',
        fetcher: getRecentlyAdded,
    },
    'most-played': {
        title: 'Most Played',
        icon: TrendingUp,
        gradient: 'from-purple-500 to-violet-500',
        description: 'Your all-time hits',
        fetcher: getMostPlayed,
    },
};

export function SmartPlaylistDetail() {
    const { type } = useParams<{ type: SmartPlaylistType }>();
    const navigate = useNavigate();
    const { playSong, setQueue, currentSong } = usePlayerStore();
    const [playlistModalSong, setPlaylistModalSong] = useState<number | null>(null);

    const config = type ? playlistConfig[type as SmartPlaylistType] : null;
    const Icon = config?.icon || Heart;

    const { data: songs, isLoading, refetch } = useQuery({
        queryKey: ['smart-playlist', type],
        queryFn: () => config?.fetcher() || Promise.resolve([]),
        enabled: !!config,
    });

    const handlePlayAll = () => {
        if (songs && songs.length > 0) {
            setQueue(songs);
            usePlayerStore.setState({ isShuffle: false });
            playSong(songs[0]);
        }
    };

    const handleShuffleAll = () => {
        if (songs && songs.length > 0) {
            const shuffled = [...songs].sort(() => Math.random() - 0.5);
            usePlayerStore.setState({ isShuffle: true, queue: shuffled, originalQueue: songs });
            playSong(shuffled[0]);
        }
    };

    const handlePlay = (song: Song) => {
        if (songs) setQueue(songs);
        playSong(song);
    };

    const handleToggleFavorite = async (songId: number, e: React.MouseEvent) => {
        e.stopPropagation();
        await toggleFavorite(songId);
        refetch();
    };

    if (!config) {
        return <div className="p-12 text-center text-apple-subtext">Playlist not found</div>;
    }

    return (
        <div className="h-full flex flex-col bg-apple-gray overflow-hidden">
            {/* Header with gradient background */}
            <div className={cn("p-8 pb-6 flex items-end gap-6 border-b border-gray-200 dark:border-white/10 flex-shrink-0 bg-gradient-to-br", config.gradient)}>
                <div className="w-32 h-32 sm:w-40 sm:h-40 bg-white/20 backdrop-blur rounded-xl shadow-xl flex items-center justify-center">
                    <Icon size={48} className="text-white" />
                </div>
                <div className="flex-1 mb-1">
                    <button onClick={() => navigate(-1)} className="text-white/80 hover:text-white text-sm font-medium flex items-center gap-1 mb-2">
                        <ArrowLeft size={16} /> Back
                    </button>
                    <h5 className="text-xs font-bold text-white/80 uppercase tracking-widest mb-1">Smart Playlist</h5>
                    <h1 className="text-3xl sm:text-5xl font-bold text-white mb-3">{config.title}</h1>
                    <div className="flex items-center gap-3 text-white/80 text-sm font-medium">
                        <span>{songs?.length || 0} Songs</span>
                        <span>•</span>
                        <span>{config.description}</span>
                        {songs && songs.length > 0 && (
                            <div className="flex gap-2 ml-4">
                                <button
                                    onClick={handlePlayAll}
                                    className="px-4 py-1.5 bg-white text-gray-900 rounded-full font-semibold flex items-center gap-2 hover:opacity-90 transition text-sm shadow-lg"
                                >
                                    <Play size={14} fill="currentColor" /> Play
                                </button>
                                <button
                                    onClick={handleShuffleAll}
                                    className="px-4 py-1.5 bg-white/20 text-white rounded-full font-semibold flex items-center gap-2 hover:bg-white/30 transition text-sm"
                                >
                                    <Shuffle size={14} /> Shuffle
                                </button>
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* Song List */}
            <div className="flex-1 overflow-y-auto pb-32">
                {isLoading ? (
                    <SongListSkeleton count={10} />
                ) : songs?.length === 0 ? (
                    <div className="p-12 text-center text-apple-subtext">
                        <Icon size={48} className="mx-auto mb-4 opacity-30" />
                        <p>No songs in this playlist yet.</p>
                        <p className="text-sm mt-1">
                            {type === 'favorites' && 'Click the heart icon on songs to add them here.'}
                            {type === 'recently-added' && 'Add music to your library to see it here.'}
                            {type === 'most-played' && 'Start playing songs to see your favorites.'}
                        </p>
                    </div>
                ) : (
                    <div className="w-full">
                        <div className="flex items-center px-8 py-2 text-xs font-semibold text-apple-subtext uppercase tracking-wider sticky top-0 bg-apple-gray/95 backdrop-blur z-10">
                            <div className="w-8 text-center">#</div>
                            <div className="w-8"></div>
                            <div className="w-12"></div>
                            <div className="flex-1 pl-4">Title</div>
                            <div className="w-1/4 hidden md:block">Artist</div>
                            {type === 'most-played' && <div className="w-16 text-center hidden sm:block">Plays</div>}
                            <div className="w-16 text-right"><Clock size={14} className="ml-auto" /></div>
                            <div className="w-10"></div>
                        </div>

                        {songs?.map((song, index) => {
                            const isActive = currentSong?.id === song.id;
                            return (
                                <div
                                    key={song.id}
                                    className={cn(
                                        "flex items-center px-8 py-2 hover:bg-gray-200/50 dark:hover:bg-white/5 cursor-pointer group/row transition-colors border-b border-gray-100 dark:border-white/5 h-14",
                                        isActive && "bg-gray-200 dark:bg-white/10"
                                    )}
                                    onClick={() => handlePlay(song)}
                                >
                                    <div className="w-8 text-center text-sm text-apple-subtext relative flex-shrink-0">
                                        <span className={cn("group-hover/row:hidden", isActive && "text-apple-accent font-bold")}>{index + 1}</span>
                                        <span className="hidden group-hover/row:flex items-center justify-center absolute inset-0">
                                            <Play size={12} fill="currentColor" className={isActive ? "text-apple-accent" : "text-apple-text"} />
                                        </span>
                                    </div>

                                    <div className="w-8 flex items-center justify-center">
                                        <button
                                            onClick={(e) => handleToggleFavorite(song.id, e)}
                                            className={cn(
                                                "p-1 rounded transition",
                                                song.rating === 5 ? "text-pink-500" : "text-gray-400 hover:text-pink-500"
                                            )}
                                        >
                                            <Heart size={14} fill={song.rating === 5 ? "currentColor" : "none"} />
                                        </button>
                                    </div>

                                    <div className="w-12 flex-shrink-0 flex justify-center">
                                        <div className="w-9 h-9 rounded overflow-hidden bg-gray-200 dark:bg-white/10 relative border border-black/5">
                                            {song.album_id ? (
                                                <img src={getCoverUrl(song.album_id)} className="w-full h-full object-cover" loading="lazy" />
                                            ) : <div className="w-full h-full bg-gray-300 dark:bg-white/10" />}
                                        </div>
                                    </div>

                                    <div className="flex-1 min-w-0 pl-4 pr-4">
                                        <div className={cn("font-medium truncate text-sm", isActive ? "text-apple-accent" : "text-apple-text")}>
                                            {song.title}
                                        </div>
                                        <div className="md:hidden text-[10px] text-apple-subtext truncate">{song.artist}</div>
                                    </div>

                                    <div className="w-1/4 hidden md:block text-sm text-apple-subtext truncate">
                                        <ArtistLink name={song.artist} />
                                    </div>

                                    {type === 'most-played' && (
                                        <div className="w-16 text-center text-sm text-apple-subtext hidden sm:block">
                                            {song.play_count || 0}
                                        </div>
                                    )}

                                    <div className="w-16 text-right text-sm text-apple-subtext tabular-nums">{formatTime(song.duration)}</div>

                                    <div className="w-10 flex items-center justify-center" onClick={(e) => e.stopPropagation()}>
                                        <SongContextMenu
                                            song={song}
                                            onAddToPlaylist={() => setPlaylistModalSong(song.id)}
                                        />
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                )}
            </div>

            {playlistModalSong && <AddToPlaylistModal songIds={[playlistModalSong]} onClose={() => setPlaylistModalSong(null)} />}
        </div>
    );
}
