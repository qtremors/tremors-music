import { useSearchParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { searchLibrary, getCoverUrl } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { formatTime, cn } from '../lib/utils';
import { Music, Search } from 'lucide-react';

export function SearchPage() {
  const [searchParams] = useSearchParams();
  const query = searchParams.get('q') || '';
  
  const { data: songs, isLoading } = useQuery({
    queryKey: ['search', query],
    queryFn: () => searchLibrary(query),
    enabled: !!query,
  });

  const { playSong, setQueue, currentSong } = usePlayerStore();

  const handlePlay = (song: any) => {
    if (songs) setQueue(songs);
    playSong(song);
  };

  return (
    <div className="h-full flex flex-col bg-apple-gray">
      <header className="h-16 flex items-center px-8 border-b border-gray-200 dark:border-white/10 flex-shrink-0">
        <h2 className="text-xl font-semibold text-apple-text">Search</h2>
      </header>

      <main className="flex-1 overflow-y-auto p-4">
        {!query ? (
          <div className="h-full flex flex-col items-center justify-center text-apple-subtext opacity-50">
            <Search size={64} className="mb-4" />
            <p>Type in the sidebar to search</p>
          </div>
        ) : isLoading ? (
          <div className="p-12 text-center text-apple-subtext">Searching...</div>
        ) : !songs || songs.length === 0 ? (
          <div className="p-12 text-center text-apple-subtext">No results found for "{query}"</div>
        ) : (
          <div className="space-y-1">
             {songs.map((song, _) => {
               const isActive = currentSong?.id === song.id;
               return (
                 <div 
                   key={song.id}
                   className={cn(
                     "flex items-center px-4 py-2 hover:bg-gray-200/50 dark:hover:bg-white/5 cursor-pointer group rounded-lg transition-colors h-14",
                     isActive && "bg-gray-200 dark:bg-white/10"
                   )}
                   onClick={() => handlePlay(song)}
                 >
                   {/* Art */}
                   <div className="w-10 h-10 rounded overflow-hidden bg-gray-200 dark:bg-white/10 relative flex-shrink-0 border border-black/5">
                      {song.album_id ? (
                        <img src={getCoverUrl(song.album_id)} className="w-full h-full object-cover" />
                      ) : <Music size={14} className="m-auto text-gray-400" />}
                   </div>

                   {/* Info */}
                   <div className="flex-1 min-w-0 px-4">
                      <div className={cn("font-medium truncate text-sm", isActive ? "text-apple-accent" : "text-apple-text")}>{song.title}</div>
                      <div className="text-xs text-apple-subtext truncate">{song.artist} • {song.album?.title}</div>
                   </div>

                   {/* Duration */}
                   <div className="text-xs text-apple-subtext font-variant-numeric tabular-nums">{formatTime(song.duration)}</div>
                 </div>
               )
             })}
          </div>
        )}
      </main>
    </div>
  );
}