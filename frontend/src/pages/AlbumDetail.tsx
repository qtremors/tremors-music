import { useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { getAlbum, getAlbumSongs, getCoverUrl } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { formatTime, cn, shuffleArray } from '../lib/utils';
import { Play, ArrowLeft, Clock, Shuffle } from 'lucide-react';
import { AlbumDetailSkeleton } from '../components/Skeletons';
import { ArtistLink } from '../components/ContextMenu';

export function AlbumDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { playSong, setQueue, currentSong } = usePlayerStore();

  const { data: album } = useQuery({
    queryKey: ['album', id],
    queryFn: () => getAlbum(Number(id)),
    enabled: !!id,
  });

  const { data: rawSongs } = useQuery({
    queryKey: ['album-songs', id],
    queryFn: () => getAlbumSongs(Number(id)),
    enabled: !!id,
  });

  // Sort songs by disc_number, then track_number for multi-disc albums
  const songs = useMemo(() => {
    if (!rawSongs) return undefined;
    return [...rawSongs].sort((a, b) => {
      const discA = a.disc_number || 1;
      const discB = b.disc_number || 1;
      if (discA !== discB) return discA - discB;
      const trackA = a.track_number || 0;
      const trackB = b.track_number || 0;
      return trackA - trackB;
    });
  }, [rawSongs]);

  const handlePlayAlbum = () => {
    if (songs && songs.length > 0) {
      setQueue(songs);
      usePlayerStore.setState({ isShuffle: false });
      playSong(songs[0]);
    }
  };

  const handleShuffleAlbum = () => {
    if (songs && songs.length > 0) {
      const shuffled = shuffleArray(songs);
      usePlayerStore.setState({ isShuffle: true, queue: shuffled, originalQueue: songs });
      playSong(shuffled[0]);
    }
  };

  if (!album) return <AlbumDetailSkeleton />;

  return (
    <div className="h-full flex flex-col overflow-y-auto bg-apple-gray pb-32">
      {/* Header / Hero Section */}
      <div className="relative w-full h-80 overflow-hidden flex items-end p-8 group">
        {/* Blurred Background */}
        <div
          className="absolute inset-0 bg-cover bg-center blur-3xl opacity-50 scale-110 dark:opacity-30"
          style={{ backgroundImage: `url(${getCoverUrl(album.id)})` }}
        />
        <div className="absolute inset-0 bg-gradient-to-t from-apple-gray via-transparent to-transparent" />

        <button
          onClick={() => navigate(-1)}
          className="absolute top-6 left-6 p-2 bg-white/20 backdrop-blur-md rounded-full text-apple-text hover:bg-white/40 transition z-20"
        >
          <ArrowLeft size={20} />
        </button>

        <div className="relative z-10 flex items-end gap-8 w-full">
          {/* Album Art Shadowed */}
          <img
            src={getCoverUrl(album.id)}
            className="w-48 h-48 rounded-lg shadow-2xl object-cover"
            alt={album.title}
          />
          <div className="flex-1 mb-2">
            <h5 className="text-apple-accent font-bold text-sm uppercase tracking-wide mb-2">Album</h5>
            <h1 className="text-4xl md:text-5xl font-bold text-apple-text mb-2 tracking-tight">{album.title}</h1>
            <div className="flex items-center gap-2 text-apple-subtext font-medium text-lg">
              <ArtistLink name={album.artist} className="text-apple-text" />
              <span>•</span>
              <span>{songs?.length} Songs</span>
            </div>
          </div>

          <div className="flex gap-3 mb-2">
            <button
              onClick={handlePlayAlbum}
              className="w-14 h-14 rounded-full bg-apple-accent text-white flex items-center justify-center shadow-lg hover:scale-105 transition"
            >
              <Play size={28} fill="currentColor" className="ml-1" />
            </button>
            <button
              onClick={handleShuffleAlbum}
              className="w-14 h-14 rounded-full bg-white/20 backdrop-blur-md text-white flex items-center justify-center shadow-lg hover:bg-white/30 transition"
            >
              <Shuffle size={24} />
            </button>
          </div>
        </div>
      </div>

      {/* Song List */}
      <div className="px-8 mt-4">
        <div className="flex items-center px-4 py-2 text-xs font-semibold text-apple-subtext border-b border-gray-200 dark:border-white/10 uppercase tracking-wider mb-2">
          <div className="w-10 text-center">#</div>
          <div className="flex-1">Title</div>
          <div className="w-20 text-right"><Clock size={14} className="ml-auto" /></div>
        </div>

        <div className="space-y-1">
          {songs?.map((song, index) => {
            const isCurrent = currentSong?.id === song.id;
            const trackNum = song.track_number || index + 1;
            return (
              <div
                key={song.id}
                onClick={() => {
                  setQueue(songs);
                  playSong(song);
                }}
                className={cn(
                  "flex items-center px-4 py-3 rounded-lg cursor-pointer group transition-colors hover:bg-gray-200/50 dark:hover:bg-white/5",
                  isCurrent && "bg-gray-200 dark:bg-white/10"
                )}
              >
                <div className="w-10 text-center text-sm text-apple-subtext font-medium relative">
                  {isCurrent ? (
                    <Play size={12} fill="currentColor" className="text-apple-accent mx-auto" />
                  ) : (
                    <>
                      <span className="group-hover:hidden">{trackNum}</span>
                      <Play size={12} fill="currentColor" className="hidden group-hover:block mx-auto text-apple-text" />
                    </>
                  )}
                </div>

                <div className={cn("flex-1 font-medium text-sm", isCurrent ? "text-apple-accent" : "text-apple-text")}>
                  {song.title}
                </div>

                <div className="w-20 text-right text-sm text-apple-subtext font-variant-numeric tabular-nums">
                  {formatTime(song.duration)}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}