import { useParams, useNavigate } from 'react-router-dom';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { getPlaylistSongs, getPlaylists, deletePlaylist, getCoverUrl } from '../lib/api';
import { usePlayerStore } from '../stores/playerStore';
import { useThemeStore } from '../stores/themeStore';
import { useToastStore } from '../stores/toastStore';
import { formatTime, cn, shuffleArray } from '../lib/utils';
import { Play, Trash2, Music, ListMusic, Shuffle } from 'lucide-react';
import { SongListSkeleton } from '../components/Skeletons';
import { useConfirm } from '../stores/confirmStore';

export function PlaylistDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { addToast } = useToastStore();

  const { playSong, setQueue, currentSong } = usePlayerStore();
  const { showSongListArt } = useThemeStore();

  // 1. Get Playlist Metadata (Name)
  const { data: playlists } = useQuery({ queryKey: ['playlists'], queryFn: getPlaylists });
  const playlistInfo = playlists?.find(p => p.id === Number(id));

  // 2. Get Songs
  const { data: songs, isLoading } = useQuery({
    queryKey: ['playlist-songs', id],
    queryFn: () => getPlaylistSongs(id!),
    enabled: !!id,
  });

  const handlePlay = (song: any) => {
    if (songs) setQueue(songs);
    playSong(song);
  };

  const handlePlayAll = () => {
    if (songs && songs.length > 0) {
      setQueue(songs);
      usePlayerStore.setState({ isShuffle: false });
      playSong(songs[0]);
    }
  };

  const handleShuffleAll = () => {
    if (songs && songs.length > 0) {
      const shuffled = shuffleArray(songs);
      usePlayerStore.setState({ isShuffle: true, queue: shuffled, originalQueue: songs });
      playSong(shuffled[0]);
    }
  };

  const confirm = useConfirm();

  const handleDeletePlaylist = async () => {
    const confirmed = await confirm({
      title: 'Delete Playlist',
      message: `Are you sure you want to delete "${playlistInfo?.name}"? This action cannot be undone.`,
      confirmText: 'Delete',
      variant: 'danger',
    });

    if (confirmed) {
      await deletePlaylist(Number(id));
      queryClient.invalidateQueries({ queryKey: ['playlists'] });
      addToast('Playlist deleted');
      navigate('/');
    }
  };

  if (!playlistInfo) return <div className="p-8 text-apple-subtext">Playlist not found</div>;

  return (
    <div className="h-full flex flex-col bg-apple-gray overflow-hidden">

      {/* Header Section */}
      <div className="p-8 pb-6 flex items-end gap-6 border-b border-gray-200 dark:border-white/10 flex-shrink-0 bg-apple-gray z-10">
        <div className="w-32 h-32 sm:w-40 sm:h-40 bg-gradient-to-br from-pink-500 to-orange-400 rounded-lg shadow-xl flex items-center justify-center text-white">
          {/* Use cover of first song if available, else icon */}
          {songs && songs.length > 0 && songs[0].album_id ? (
            <img src={getCoverUrl(songs[0].album_id)} className="w-full h-full object-cover rounded-lg" />
          ) : (
            <ListMusic size={48} />
          )}
        </div>
        <div className="flex-1 mb-1">
          <h5 className="text-xs font-bold text-apple-accent uppercase tracking-widest mb-1">Playlist</h5>
          <h1 className="text-3xl sm:text-5xl font-bold text-apple-text mb-3">{playlistInfo.name}</h1>
          <div className="flex items-center gap-3 text-apple-subtext text-sm font-medium">
            <span>{songs?.length || 0} Songs</span>
            {songs && songs.length > 0 && (
              <div className="flex gap-2">
                <button
                  onClick={handlePlayAll}
                  className="px-4 py-1.5 bg-apple-accent text-white rounded-full font-semibold flex items-center gap-2 hover:opacity-90 transition text-sm shadow-lg"
                >
                  <Play size={14} fill="currentColor" /> Play
                </button>
                <button
                  onClick={handleShuffleAll}
                  className="px-4 py-1.5 bg-gray-200 dark:bg-white/10 text-apple-text rounded-full font-semibold flex items-center gap-2 hover:bg-gray-300 dark:hover:bg-white/20 transition text-sm"
                >
                  <Shuffle size={14} /> Shuffle
                </button>
              </div>
            )}
          </div>
        </div>
        <button
          onClick={handleDeletePlaylist}
          className="p-2 text-apple-subtext hover:text-red-500 transition mb-2 bg-gray-100 dark:bg-white/5 rounded-lg"
          title="Delete Playlist"
        >
          <Trash2 size={20} />
        </button>
      </div>

      {/* Song List Table */}
      <div className="flex-1 overflow-y-auto pb-32">
        {isLoading ? (
          <SongListSkeleton count={8} />
        ) : songs?.length === 0 ? (
          <div className="p-12 text-center text-apple-subtext">This playlist is empty. Add songs from the Library.</div>
        ) : (
          <div className="w-full">
            <div className="flex items-center px-8 py-2 text-xs font-semibold text-apple-subtext uppercase tracking-wider sticky top-0 bg-apple-gray/95 backdrop-blur z-10">
              <div className="w-8 text-center">#</div>
              {showSongListArt && <div className="w-12"></div>}
              <div className="flex-1 pl-4">Title</div>
              <div className="w-1/3 hidden md:block">Artist</div>
              <div className="w-16 text-right">Time</div>
            </div>

            {songs?.map((song, index) => {
              const isActive = currentSong?.id === song.id;
              return (
                <div
                  key={`${song.id}-${index}`} // index key needed as duplicates allowed in playlists
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

                  {showSongListArt && (
                    <div className="w-12 flex-shrink-0 flex justify-center">
                      <div className="w-9 h-9 rounded overflow-hidden bg-gray-200 dark:bg-white/10 relative border border-black/5">
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
                  </div>
                  <div className="w-1/3 hidden md:block text-sm text-apple-subtext truncate">{song.artist}</div>
                  <div className="w-16 text-right text-sm text-apple-subtext tabular-nums">{formatTime(song.duration)}</div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}