import { useSearchParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { getCoverUrl } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { formatTime, cn } from '../lib/utils';
import { Music, Search, Disc, User, Play } from 'lucide-react';
import api from '../lib/api';
import { SearchPageSkeleton } from '../components/Skeletons';
import { ArtistLink } from '../components/ContextMenu';

interface SearchResults {
  songs: any[];
  albums: any[];
  artists: { name: string }[];
  bestMatchType?: 'song' | 'album' | 'artist' | null;
  scores?: {
    song: number;
    album: number;
    artist: number;
  };
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
      const data = res.data || {};
      return {
        songs: Array.isArray(data.songs) ? data.songs : [],
        albums: Array.isArray(data.albums) ? data.albums : [],
        artists: Array.isArray(data.artists) ? data.artists : [],
        bestMatchType: data.bestMatchType,
        scores: data.scores,
      };
    },
    enabled: !!query,
  });

  const { playSong, setQueue, currentSong } = usePlayerStore();

  const handlePlaySong = (song: any) => {
    if (results?.songs) setQueue(results.songs);
    playSong(song);
  };

  const handlePlayAlbum = (album: any) => {
    navigate(`/albums/${album.id}`);
  };

  const handleViewArtist = (artist: { name: string }) => {
    navigate(`/artists/${encodeURIComponent(artist.name)}`);
  };

  // Determine top result using backend's bestMatchType
  let topResult: any = null;
  let topResultType: 'song' | 'album' | 'artist' | null = null;

  if (results && query) {
    const bestType = results.bestMatchType;

    if (bestType === 'song' && results.songs.length > 0) {
      topResult = results.songs[0];
      topResultType = 'song';
    } else if (bestType === 'artist' && results.artists.length > 0) {
      topResult = results.artists[0];
      topResultType = 'artist';
    } else if (bestType === 'album' && results.albums.length > 0) {
      topResult = results.albums[0];
      topResultType = 'album';
    }
    // Fallback logic if bestMatchType is not set (backwards compatibility)
    else if (results.songs.length > 0) {
      topResult = results.songs[0];
      topResultType = 'song';
    } else if (results.albums.length > 0) {
      topResult = results.albums[0];
      topResultType = 'album';
    } else if (results.artists.length > 0) {
      topResult = results.artists[0];
      topResultType = 'artist';
    }
  }

  // Get remaining items after top result
  const remainingSongs = topResultType === 'song' ? results?.songs.slice(1) : results?.songs;
  const remainingAlbums = topResultType === 'album' ? results?.albums.slice(1) : results?.albums;
  const remainingArtists = topResultType === 'artist' ? results?.artists.slice(1) : results?.artists;

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
          <SearchPageSkeleton />
        ) : !results || (results.songs.length === 0 && results.albums.length === 0 && results.artists.length === 0) ? (
          <div className="p-12 text-center text-apple-subtext">No results found for "{query}"</div>
        ) : (
          <div className="space-y-8">
            {/* Top Result */}
            {topResult && (
              <div>
                <h3 className="text-lg font-semibold text-apple-text mb-4">Top Result</h3>

                {/* Song Top Result */}
                {topResultType === 'song' && (
                  <div
                    onClick={() => handlePlaySong(topResult)}
                    className="flex items-center gap-6 p-6 rounded-xl bg-gradient-to-br from-apple-accent/10 to-transparent hover:from-apple-accent/20 cursor-pointer group transition-all border border-apple-accent/20"
                  >
                    <div className="w-32 h-32 rounded-lg overflow-hidden shadow-xl flex-shrink-0 relative">
                      {topResult.album_id ? (
                        <img
                          src={getCoverUrl(topResult.album_id)}
                          alt={topResult.title}
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                        />
                      ) : (
                        <div className="w-full h-full bg-gradient-to-br from-apple-accent/30 to-apple-accent/10 flex items-center justify-center">
                          <Music size={48} className="text-apple-accent" />
                        </div>
                      )}
                      <div className="absolute inset-0 bg-black/30 flex items-center justify-center opacity-0 group-hover:opacity-100 transition">
                        <div className="w-12 h-12 rounded-full bg-apple-accent flex items-center justify-center">
                          <Play size={20} fill="white" className="text-white ml-0.5" />
                        </div>
                      </div>
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-4xl font-bold text-apple-text mb-2 truncate group-hover:text-apple-accent transition">
                        {topResult.title}
                      </div>
                      <div className="text-lg text-apple-subtext mb-3 truncate">
                        <ArtistLink name={topResult.artist} />
                      </div>
                      <div className="flex items-center gap-3">
                        <div className="inline-flex items-center gap-2 px-3 py-1 bg-white dark:bg-gray-800 rounded-full text-xs font-medium text-apple-subtext">
                          <Music size={14} />
                          Song
                        </div>
                        <span className="text-sm text-apple-subtext">{formatTime(topResult.duration)}</span>
                      </div>
                    </div>
                  </div>
                )}

                {/* Album Top Result */}
                {topResultType === 'album' && (
                  <div
                    onClick={() => handlePlayAlbum(topResult)}
                    className="flex items-center gap-6 p-6 rounded-xl bg-gradient-to-br from-apple-accent/10 to-transparent hover:from-apple-accent/20 cursor-pointer group transition-all border border-apple-accent/20"
                  >
                    <div className="w-32 h-32 rounded-lg overflow-hidden shadow-xl flex-shrink-0">
                      <img
                        src={getCoverUrl(topResult.id)}
                        alt={topResult.title}
                        className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                      />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-4xl font-bold text-apple-text mb-2 truncate group-hover:text-apple-accent transition">
                        {topResult.title}
                      </div>
                      <div className="text-lg text-apple-subtext mb-3 truncate">
                        <ArtistLink name={topResult.artist} />
                      </div>
                      <div className="inline-flex items-center gap-2 px-3 py-1 bg-white dark:bg-gray-800 rounded-full text-xs font-medium text-apple-subtext">
                        <Disc size={14} />
                        Album
                      </div>
                    </div>
                  </div>
                )}

                {/* Artist Top Result */}
                {topResultType === 'artist' && (
                  <div
                    className="flex items-center gap-6 p-6 rounded-xl bg-gradient-to-br from-apple-accent/10 to-transparent hover:from-apple-accent/20 cursor-pointer group transition-all border border-apple-accent/20"
                    onClick={() => handleViewArtist(topResult)}
                  >
                    <div className="w-32 h-32 rounded-full bg-gradient-to-br from-apple-accent/30 to-apple-accent/10 flex items-center justify-center shadow-xl flex-shrink-0 group-hover:scale-105 transition-transform">
                      <User size={64} className="text-apple-accent" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-4xl font-bold text-apple-text mb-2 truncate group-hover:text-apple-accent transition">
                        {topResult.name}
                      </div>
                      <div className="inline-flex items-center gap-2 px-3 py-1 bg-white dark:bg-gray-800 rounded-full text-xs font-medium text-apple-subtext mt-2">
                        <User size={14} />
                        Artist
                      </div>
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* Songs Section */}
            {remainingSongs && remainingSongs.length > 0 && (
              <div>
                <h3 className="text-lg font-semibold text-apple-text mb-4 flex items-center gap-2">
                  <Music size={20} className="text-apple-accent" />
                  Songs
                </h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                  {remainingSongs.slice(0, 8).map((song) => {
                    const isActive = currentSong?.id === song.id;
                    return (
                      <div
                        key={song.id}
                        className={cn(
                          "flex items-center gap-3 px-4 py-3 rounded-lg cursor-pointer group transition-all hover:bg-gray-200/50 dark:hover:bg-white/5",
                          isActive && "bg-gray-200 dark:bg-white/10"
                        )}
                        onClick={() => handlePlaySong(song)}
                      >
                        {/* Album Art */}
                        <div className="w-12 h-12 rounded overflow-hidden bg-gray-200 dark:bg-white/10 relative flex-shrink-0 border border-black/5">
                          {song.album_id ? (
                            <img src={getCoverUrl(song.album_id)} className="w-full h-full object-cover" alt="" />
                          ) : <Music size={16} className="m-auto text-gray-400" />}
                          <div className="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition">
                            <Play size={16} fill="white" className="text-white" />
                          </div>
                        </div>

                        {/* Info */}
                        <div className="flex-1 min-w-0">
                          <div className={cn("font-medium truncate text-sm", isActive ? "text-apple-accent" : "text-apple-text")}>
                            {song.title}
                          </div>
                          <div className="text-xs text-apple-subtext truncate">
                            <ArtistLink name={song.artist} />
                          </div>
                        </div>

                        {/* Duration */}
                        <div className="text-xs text-apple-subtext font-variant-numeric tabular-nums">
                          {formatTime(song.duration)}
                        </div>
                      </div>
                    );
                  })}
                </div>
                {remainingSongs.length > 8 && (
                  <div className="text-xs text-apple-subtext mt-3 text-center">
                    +{remainingSongs.length - 8} more songs
                  </div>
                )}
              </div>
            )}

            {/* Albums Section */}
            {remainingAlbums && remainingAlbums.length > 0 && (
              <div>
                <h3 className="text-lg font-semibold text-apple-text mb-4 flex items-center gap-2">
                  <Disc size={20} className="text-apple-accent" />
                  Albums
                </h3>
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
                  {remainingAlbums.map((album) => (
                    <div
                      key={album.id}
                      onClick={() => handlePlayAlbum(album)}
                      className="group cursor-pointer"
                    >
                      <div className="aspect-square rounded-lg overflow-hidden bg-gray-200 dark:bg-white/10 mb-3 shadow-md group-hover:shadow-xl transition-all relative">
                        <img
                          src={getCoverUrl(album.id)}
                          alt={album.title}
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                        />
                        <div className="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition">
                          <div className="w-12 h-12 rounded-full bg-apple-accent flex items-center justify-center">
                            <Play size={20} fill="white" className="text-white ml-0.5" />
                          </div>
                        </div>
                      </div>
                      <h4 className="font-medium text-sm text-apple-text truncate group-hover:text-apple-accent transition">
                        {album.title}
                      </h4>
                      <p className="text-xs text-apple-subtext truncate">
                        <ArtistLink name={album.artist} />
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Artists Section */}
            {remainingArtists && remainingArtists.length > 0 && (
              <div>
                <h3 className="text-lg font-semibold text-apple-text mb-4 flex items-center gap-2">
                  <User size={20} className="text-apple-accent" />
                  Artists
                </h3>
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
                  {remainingArtists.map((artist, idx) => (
                    <div
                      key={idx}
                      onClick={() => handleViewArtist(artist)}
                      className="flex flex-col items-center p-4 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 hover:border-apple-accent dark:hover:border-apple-accent cursor-pointer transition group"
                    >
                      <div className="w-20 h-20 rounded-full bg-gradient-to-br from-apple-accent/20 to-apple-accent/5 flex items-center justify-center mb-3 group-hover:scale-110 transition-transform">
                        <User size={32} className="text-apple-accent" />
                      </div>
                      <div className="font-medium text-sm text-apple-text text-center truncate w-full group-hover:text-apple-accent transition">
                        {artist.name}
                      </div>
                      <div className="text-xs text-apple-subtext mt-1">Artist</div>
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