import { useSearchParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { getCoverUrl } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { formatTime, cn } from '../lib/utils';
import { Music, Search, Disc, User } from 'lucide-react';
import api from '../lib/api';

interface SearchResults {
  songs: any[];
  albums: any[];
  artists: { name: string }[];
}

export function SearchPage() {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const query = searchParams.get('q') || '';

  const { data: results, isLoading } = useQuery<SearchResults>({
    queryKey: ['search', query],
    queryFn: async () => {
      if (!query) return { songs: [], albums: [], artists: [] };
      const res = await api.get(`/library/search?q=${encodeURIComponent(query)}`);
      return res.data;
    },
    enabled: !!query,
  });

  const { playSong, setQueue, currentSong } = usePlayerStore();

  const handlePlaySong = (song: any) => {
    if (results?.songs) setQueue(results.songs);
    playSong(song);
  };

  const handlePlayAlbum = (album: any) => {
    // Navigate to album detail page
    navigate(`/album/${album.id}`);
  };

  return (
    <div className="h-full flex flex-col bg-apple-gray">
      <header className="h-16 flex items-center px-8 border-b border-gray-200 dark:border-white/10 flex-shrink-0">
        <h2 className="text-xl font-semibold text-apple-text">Search</h2>
      </header>

      <main className="flex-1 overflow-y-auto p-8 pb-32">
        {!query ? (
          <div className="h-full flex flex-col items-center justify-center text-apple-subtext opacity-50">
            <Search size={64} className="mb-4" />
            <p>Type in the sidebar to search</p>
          </div>
        ) : isLoading ? (
          <div className="p-12 text-center text-apple-subtext">Searching...</div>
        ) : !results || (results.songs.length === 0 && results.albums.length === 0 && results.artists.length === 0) ? (
          <div className="p-12 text-center text-apple-subtext">No results found for "{query}"</div>
        ) : (
          <div className="space-y-8">
            {/* Songs Section */}
            {results.songs.length > 0 && (
              <div>
                <h3 className="text-lg font-semibold text-apple-text mb-4 flex items-center gap-2">
                  <Music size={20} className="text-apple-accent" />
                  Songs ({results.songs.length})
                </h3>
                <div className="space-y-1">
                  {results.songs.map((song) => {
                    const isActive = currentSong?.id === song.id;
                    return (
                      <div
                        key={song.id}
                        className={cn(
                          "flex items-center px-4 py-2 hover:bg-gray-200/50 dark:hover:bg-white/5 cursor-pointer group rounded-lg transition-colors h-14",
                          isActive && "bg-gray-200 dark:bg-white/10"
                        )}
                        onClick={() => handlePlaySong(song)}
                      >
                        {/* Art */}
                        <div className="w-10 h-10 rounded overflow-hidden bg-gray-200 dark:bg-white/10 relative flex-shrink-0 border border-black/5">
                          {song.album_id ? (
                            <img src={getCoverUrl(song.album_id)} className="w-full h-full object-cover" alt="" />
                          ) : <Music size={14} className="m-auto text-gray-400" />}
                        </div>

                        {/* Info */}
                        <div className="flex-1 min-w-0 px-4">
                          <div className={cn("font-medium truncate text-sm", isActive ? "text-apple-accent" : "text-apple-text")}>
                            {song.title}
                          </div>
                          <div className="text-xs text-apple-subtext truncate">{song.artist}</div>
                        </div>

                        {/* Duration */}
                        <div className="text-xs text-apple-subtext font-variant-numeric tabular-nums">
                          {formatTime(song.duration)}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}

            {/* Albums Section */}
            {results.albums.length > 0 && (
              <div>
                <h3 className="text-lg font-semibold text-apple-text mb-4 flex items-center gap-2">
                  <Disc size={20} className="text-apple-accent" />
                  Albums ({results.albums.length})
                </h3>
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
                  {results.albums.map((album) => (
                    <div
                      key={album.id}
                      onClick={() => handlePlayAlbum(album)}
                      className="group cursor-pointer"
                    >
                      <div className="aspect-square rounded-lg overflow-hidden bg-gray-200 dark:bg-white/10 mb-3 shadow-md group-hover:shadow-xl transition-shadow">
                        <img
                          src={getCoverUrl(album.id)}
                          alt={album.title}
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                        />
                      </div>
                      <h4 className="font-medium text-sm text-apple-text truncate group-hover:text-apple-accent transition">
                        {album.title}
                      </h4>
                      <p className="text-xs text-apple-subtext truncate">{album.artist}</p>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Artists Section */}
            {results.artists.length > 0 && (
              <div>
                <h3 className="text-lg font-semibold text-apple-text mb-4 flex items-center gap-2">
                  <User size={20} className="text-apple-accent" />
                  Artists ({results.artists.length})
                </h3>
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
                  {results.artists.map((artist, idx) => (
                    <div
                      key={idx}
                      className="px-4 py-3 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 hover:border-apple-accent dark:hover:border-apple-accent cursor-pointer transition group"
                    >
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-10 rounded-full bg-apple-accent/10 flex items-center justify-center flex-shrink-0">
                          <User size={18} className="text-apple-accent" />
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="font-medium text-sm text-apple-text truncate group-hover:text-apple-accent transition">
                            {artist.name}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </main>
    </div>
  );
}