import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { getArtistWork, getCoverUrl } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { useToastStore } from '../stores/toastStore';
import { formatTime, cn, shuffleArray } from '../lib/utils';
import { Play, Shuffle, ArrowLeft, Disc, MoreVertical, ListPlus } from 'lucide-react';
import { ArtistDetailSkeleton } from '../components/Skeletons';
import { SongContextMenu, AlbumLink } from '../components/ContextMenu';
import { useState, useRef, useEffect } from 'react';
import { AddToPlaylistModal } from '../components/AddToPlaylistModal';

export function ArtistDetail() {
    const { name } = useParams<{ name: string }>();
    const navigate = useNavigate();
    const { playSong, setQueue, currentSong, addToQueue } = usePlayerStore();
    const { addToast } = useToastStore();
    const [playlistModalSong, setPlaylistModalSong] = useState<number | null>(null);

    const { data: work, isLoading } = useQuery({
        queryKey: ['artist-work', name],
        queryFn: () => getArtistWork(name || ''),
        enabled: !!name,
    });

    if (isLoading) return <ArtistDetailSkeleton />;
    if (!work || !Array.isArray(work)) return null;

    // Flatten songs for the "Play All" button
    const allSongs = work.flatMap(group => Array.isArray(group.songs) ? group.songs : []);

    const handlePlayAll = () => {
        if (allSongs.length === 0) return;
        usePlayerStore.setState({ isShuffle: false });
        setQueue(allSongs);
        playSong(allSongs[0]);
    };

    const handleShuffleAll = () => {
        if (allSongs.length === 0) return;
        const shuffled = shuffleArray(allSongs);
        usePlayerStore.setState({ isShuffle: true, queue: shuffled, originalQueue: allSongs });
        playSong(shuffled[0]);
    };

    return (
        <div className="h-full flex flex-col bg-apple-gray overflow-y-auto pb-32">

            {/* Header */}
            <div className="p-8 pb-4 flex flex-col gap-4 bg-apple-gray z-10">
                <button onClick={() => navigate(-1)} className="flex items-center gap-1 text-apple-accent font-medium w-fit hover:underline">
                    <ArrowLeft size={18} /> Artists
                </button>

                <div className="flex items-end justify-between">
                    <div>
                        <h1 className="text-4xl font-bold text-apple-text mb-1">{name}</h1>
                        <p className="text-apple-subtext text-sm font-medium">{work.length} Albums • {allSongs.length} Songs</p>
                    </div>

                    <div className="flex gap-3">
                        <button onClick={handlePlayAll} className="w-10 h-10 bg-apple-accent text-white rounded-full flex items-center justify-center shadow-md hover:opacity-90 transition">
                            <Play size={20} fill="currentColor" className="ml-1" />
                        </button>
                        <button onClick={handleShuffleAll} className="w-10 h-10 bg-gray-200 dark:bg-white/10 text-apple-text rounded-full flex items-center justify-center hover:bg-gray-300 dark:hover:bg-white/20 transition">
                            <Shuffle size={18} />
                        </button>
                    </div>
                </div>
            </div>

            <div className="px-8 space-y-12">
                {work.map((group, idx) => (
                    <AlbumSection
                        key={idx}
                        group={group}
                        idx={idx}
                        navigate={navigate}
                        playSong={playSong}
                        setQueue={setQueue}
                        addToQueue={addToQueue}
                        addToast={addToast}
                        currentSong={currentSong}
                        onAddToPlaylist={(songId) => setPlaylistModalSong(songId)}
                    />
                ))}
            </div>

            {playlistModalSong && <AddToPlaylistModal songIds={[playlistModalSong]} onClose={() => setPlaylistModalSong(null)} />}
        </div>
    );
}

// Album section with context menu
function AlbumSection({
    group,
    idx,
    navigate,
    playSong,
    setQueue,
    addToQueue,
    addToast,
    currentSong,
    onAddToPlaylist
}: {
    group: { album: any; songs: any[] };
    idx: number;
    navigate: ReturnType<typeof useNavigate>;
    playSong: (song: any) => void;
    setQueue: (songs: any[]) => void;
    addToQueue: (song: any) => void;
    addToast: (msg: string, type?: 'success' | 'error' | 'info') => void;
    currentSong: any;
    onAddToPlaylist: (songId: number) => void;
}) {
    const [showAlbumMenu, setShowAlbumMenu] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (showAlbumMenu) {
            const handleClickOutside = (e: MouseEvent) => {
                if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                    setShowAlbumMenu(false);
                }
            };
            document.addEventListener('mousedown', handleClickOutside);
            return () => document.removeEventListener('mousedown', handleClickOutside);
        }
    }, [showAlbumMenu]);

    const handlePlayAlbum = async () => {
        if (group.songs.length > 0) {
            setQueue(group.songs);
            usePlayerStore.setState({ isShuffle: false });
            playSong(group.songs[0]);
        }
        setShowAlbumMenu(false);
    };

    const handleShuffleAlbum = () => {
        if (group.songs.length > 0) {
            const shuffled = shuffleArray(group.songs);
            setQueue(shuffled);
            usePlayerStore.setState({ isShuffle: true, originalQueue: group.songs });
            playSong(shuffled[0]);
        }
        setShowAlbumMenu(false);
    };

    const handleAddAlbumToQueue = () => {
        group.songs.forEach(song => addToQueue(song));
        addToast(`Added ${group.songs.length} songs to queue`);
        setShowAlbumMenu(false);
    };

    return (
        <div className="flex flex-col md:flex-row gap-8 animate-in fade-in slide-in-from-bottom-4 duration-500" style={{ animationDelay: `${idx * 100}ms` }}>

            {/* Album Info (Left Side) */}
            <div className="md:w-48 flex-shrink-0">
                <div className="relative group">
                    {/* Clickable Album Cover */}
                    <div
                        onClick={() => group.album && navigate(`/albums/${group.album.id}`)}
                        className={cn(
                            "w-40 h-40 rounded-lg shadow-lg overflow-hidden bg-gray-200 dark:bg-white/5 relative border border-black/5",
                            group.album && "cursor-pointer"
                        )}
                    >
                        {group.album ? (
                            <>
                                <img src={getCoverUrl(group.album.id)} className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" loading="lazy" />
                                <div className="absolute inset-0 bg-black/30 flex items-center justify-center opacity-0 group-hover:opacity-100 transition">
                                    <div className="w-12 h-12 rounded-full bg-apple-accent flex items-center justify-center">
                                        <Play size={20} fill="white" className="text-white ml-0.5" />
                                    </div>
                                </div>
                            </>
                        ) : <Disc size={48} className="m-auto text-gray-400" />}
                    </div>

                    {/* Album 3-dot menu */}
                    {group.album && (
                        <div className="absolute top-2 right-2 z-10" ref={menuRef}>
                            <button
                                onClick={(e) => { e.stopPropagation(); setShowAlbumMenu(!showAlbumMenu); }}
                                className="p-1.5 rounded-full bg-black/40 hover:bg-black/60 text-white opacity-0 group-hover:opacity-100 transition"
                            >
                                <MoreVertical size={16} />
                            </button>
                            {showAlbumMenu && (
                                <div className="absolute right-0 top-8 w-48 bg-white dark:bg-[#2c2c2e] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 py-2 z-50 animate-in fade-in zoom-in-95 duration-100">
                                    <button onClick={handlePlayAlbum} className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-white/5 flex items-center gap-3 text-apple-text transition">
                                        <Play size={16} /> Play Album
                                    </button>
                                    <button onClick={handleShuffleAlbum} className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-white/5 flex items-center gap-3 text-apple-text transition">
                                        <Shuffle size={16} /> Shuffle
                                    </button>
                                    <button onClick={handleAddAlbumToQueue} className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-white/5 flex items-center gap-3 text-apple-text transition">
                                        <ListPlus size={16} /> Add to Queue
                                    </button>
                                </div>
                            )}
                        </div>
                    )}
                </div>

                <div className="mt-3">
                    {group.album ? (
                        <AlbumLink id={group.album.id} title={group.album.title} className="font-bold text-apple-text text-lg leading-tight hover:text-apple-accent" />
                    ) : (
                        <h3 className="font-bold text-apple-text text-lg leading-tight">Unknown Album</h3>
                    )}
                    {group.album?.year && <p className="text-sm text-apple-subtext mt-1">{group.album.year}</p>}
                </div>
            </div>

            {/* Songs (Right Side) */}
            <div className="flex-1 min-w-0">
                {(group.songs || []).map((song) => {
                    const isActive = currentSong?.id === song.id;
                    return (
                        <div
                            key={song.id}
                            onClick={() => { setQueue(group.songs); playSong(song); }}
                            className={cn(
                                "flex items-center px-4 py-3 rounded-lg cursor-pointer group transition-colors hover:bg-gray-200/50 dark:hover:bg-white/5 border-b border-gray-100 dark:border-white/5 last:border-0",
                                isActive && "bg-gray-200 dark:bg-white/10"
                            )}
                        >
                            <div className="w-8 text-center text-sm font-medium text-apple-subtext">
                                {isActive ? <Play size={12} fill="currentColor" className="text-apple-accent mx-auto" /> : song.track_number || '-'}
                            </div>
                            <div className="flex-1 px-4 min-w-0">
                                <div className={cn("font-medium truncate", isActive ? "text-apple-accent" : "text-apple-text")}>
                                    {song.title}
                                </div>
                            </div>
                            <div className="text-sm text-apple-subtext font-variant-numeric tabular-nums">
                                {formatTime(song.duration)}
                            </div>
                            <div className="w-8 flex justify-end" onClick={(e) => e.stopPropagation()}>
                                <SongContextMenu
                                    song={song}
                                    onAddToPlaylist={() => onAddToPlaylist(song.id)}
                                    position="left"
                                />
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}
