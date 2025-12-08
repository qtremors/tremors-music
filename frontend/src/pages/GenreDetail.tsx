import { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { getGenreSongs, getCoverUrl } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { formatTime, cn } from '../lib/utils';
import { Play, Shuffle, ArrowLeft, Music } from 'lucide-react';
import { SongListSkeleton } from '../components/Skeletons';
import { SongContextMenu, ArtistLink, AlbumLink } from '../components/ContextMenu';
import { AddToPlaylistModal } from '../components/AddToPlaylistModal';

export function GenreDetail() {
    const { name } = useParams<{ name: string }>();
    const navigate = useNavigate();
    const { playSong, setQueue, currentSong } = usePlayerStore();
    const [playlistModalSong, setPlaylistModalSong] = useState<number | null>(null);

    const decodedName = name ? decodeURIComponent(name) : '';

    const { data: songs, isLoading } = useQuery({
        queryKey: ['genre-songs', decodedName],
        queryFn: () => getGenreSongs(decodedName),
        enabled: !!decodedName,
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

    const handlePlay = (song: any) => {
        if (songs) setQueue(songs);
        playSong(song);
    };

    // Color based on genre name hash
    const colorIndex = decodedName.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0) % 10;
    const colors = [
        'from-red-500 to-orange-500',
        'from-orange-500 to-yellow-500',
        'from-yellow-500 to-green-500',
        'from-green-500 to-teal-500',
        'from-teal-500 to-cyan-500',
        'from-cyan-500 to-blue-500',
        'from-blue-500 to-indigo-500',
        'from-indigo-500 to-purple-500',
        'from-purple-500 to-pink-500',
        'from-pink-500 to-red-500',
    ];

    return (
        <div className="h-full flex flex-col bg-apple-gray overflow-hidden">
            {/* Header with gradient background */}
            <div className={cn("p-8 pb-6 flex items-end gap-6 border-b border-gray-200 dark:border-white/10 flex-shrink-0 bg-gradient-to-br", colors[colorIndex])}>
                <div className="w-32 h-32 sm:w-40 sm:h-40 bg-white/20 backdrop-blur rounded-xl shadow-xl flex items-center justify-center">
                    <Music size={48} className="text-white" />
                </div>
                <div className="flex-1 mb-1">
                    <button onClick={() => navigate(-1)} className="text-white/80 hover:text-white text-sm font-medium flex items-center gap-1 mb-2">
                        <ArrowLeft size={16} /> Genres
                    </button>
                    <h5 className="text-xs font-bold text-white/80 uppercase tracking-widest mb-1">Genre</h5>
                    <h1 className="text-3xl sm:text-5xl font-bold text-white mb-3">{decodedName}</h1>
                    <div className="flex items-center gap-3 text-white/80 text-sm font-medium">
                        <span>{songs?.length || 0} Songs</span>
                        {songs && songs.length > 0 && (
                            <div className="flex gap-2">
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
                    <div className="p-12 text-center text-apple-subtext">No songs found in this genre.</div>
                ) : (
                    <div className="w-full">
                        <div className="flex items-center px-8 py-2 text-xs font-semibold text-apple-subtext uppercase tracking-wider sticky top-0 bg-apple-gray/95 backdrop-blur z-10">
                            <div className="w-8 text-center">#</div>
                            <div className="w-12"></div>
                            <div className="flex-1 pl-4">Title</div>
                            <div className="w-1/4 hidden md:block">Artist</div>
                            <div className="w-1/4 hidden lg:block pl-4">Album</div>
                            <div className="w-16 text-right">Time</div>
                            <div className="w-10"></div>
                        </div>

                        {songs?.map((song, index) => {
                            const isActive = currentSong?.id === song.id;
                            return (
                                <div
                                    key={song.id}
                                    className={cn(
                                        "flex items-center px-8 py-2 hover:bg-gray-200/50 dark:hover:bg-white/5 cursor-pointer group transition-colors border-b border-gray-100 dark:border-white/5 h-14",
                                        isActive && "bg-gray-200 dark:bg-white/10"
                                    )}
                                    onClick={() => handlePlay(song)}
                                >
                                    <div className="w-8 text-center text-sm text-apple-subtext relative flex-shrink-0">
                                        <span className={cn("group-hover:hidden", isActive && "text-apple-accent font-bold")}>{index + 1}</span>
                                        <span className="hidden group-hover:flex items-center justify-center absolute inset-0">
                                            <Play size={12} fill="currentColor" className={isActive ? "text-apple-accent" : "text-apple-text"} />
                                        </span>
                                    </div>

                                    <div className="w-12 flex-shrink-0 flex justify-center">
                                        <div className="w-9 h-9 rounded overflow-hidden bg-gray-200 dark:bg-white/10 relative border border-black/5">
                                            {song.album_id ? (
                                                <img src={getCoverUrl(song.album_id)} className="w-full h-full object-cover" loading="lazy" />
                                            ) : <Music size={14} className="m-auto text-gray-400" />}
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

                                    <div className="w-1/4 hidden lg:block text-sm text-apple-subtext truncate pl-4">
                                        {song.album_id && (
                                            <AlbumLink id={song.album_id} title={song.album?.title || 'Unknown Album'} />
                                        )}
                                    </div>

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
