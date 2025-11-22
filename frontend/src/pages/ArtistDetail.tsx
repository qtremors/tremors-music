import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { getArtistWork, getCoverUrl } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { formatTime, cn } from '../lib/utils';
import { Play, Shuffle, ArrowLeft, MoreHorizontal, Disc } from 'lucide-react';

export function ArtistDetail() {
  const { name } = useParams<{ name: string }>();
  const navigate = useNavigate();
  const { playSong, setQueue, currentSong } = usePlayerStore();

  const { data: work, isLoading } = useQuery({
    queryKey: ['artist-work', name],
    queryFn: () => getArtistWork(name || ''),
    enabled: !!name,
  });

  if (isLoading) return <div className="p-12 text-center text-apple-subtext">Loading Artist...</div>;
  if (!work) return null;

  // Flatten songs for the "Play All" button
  const allSongs = work.flatMap(group => group.songs);

  const handlePlayAll = () => {
    if (allSongs.length === 0) return;
    usePlayerStore.setState({ isShuffle: false });
    setQueue(allSongs);
    playSong(allSongs[0]);
  };

  const handleShuffleAll = () => {
    if (allSongs.length === 0) return;
    // Randomize
    const shuffled = [...allSongs].sort(() => Math.random() - 0.5);
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
            <div key={idx} className="flex flex-col md:flex-row gap-8 animate-in fade-in slide-in-from-bottom-4 duration-500" style={{ animationDelay: `${idx * 100}ms` }}>
                
                {/* Album Info (Left Side) */}
                <div className="md:w-48 flex-shrink-0">
                    <div className="w-40 h-40 rounded-lg shadow-lg overflow-hidden bg-gray-200 dark:bg-white/5 relative border border-black/5">
                        {group.album ? (
                            <img src={getCoverUrl(group.album.id)} className="w-full h-full object-cover" loading="lazy" />
                        ) : <Disc size={48} className="m-auto text-gray-400" />}
                    </div>
                    <div className="mt-3">
                        <h3 className="font-bold text-apple-text text-lg leading-tight">{group.album?.title || "Unknown Album"}</h3>
                        <p className="text-sm text-apple-subtext mt-1">R&B/Soul • 2020</p> {/* Placeholder for genre/year if we had it */}
                    </div>
                </div>

                {/* Songs (Right Side) */}
                <div className="flex-1 min-w-0">
                    {group.songs.map((song) => {
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
                                <div className="w-8 flex justify-end opacity-0 group-hover:opacity-100 transition">
                                    <MoreHorizontal size={16} className="text-apple-subtext" />
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>
        ))}
      </div>
    </div>
  );
}