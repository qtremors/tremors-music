import { useEffect, useRef, useState } from 'react';
import { usePlayerStore } from '../stores/playerStore';
import { getStreamUrl, getCoverUrl } from '../lib/api';
import { formatTime, cn } from '../lib/utils';
import { Play, Pause, SkipBack, SkipForward, Volume2, VolumeX, Disc, ChevronUp, Shuffle, Repeat, Repeat1, ListMusic } from 'lucide-react';
import { FullScreenPlayer } from './FullScreenPlayer';
import { QueuePanel } from './player/QueuePanel';
import { useKeyboardShortcuts } from '../hooks/useKeyboardShortcuts';

export function Player() {
  const {
    currentSong, isPlaying, volume, togglePlay, playNext, playPrev,
    setVolume, setIsPlaying, repeatMode, toggleRepeat, isShuffle, toggleShuffle
  } = usePlayerStore();

  const audioRef = useRef<HTMLAudioElement>(null);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [isFullScreen, setIsFullScreen] = useState(false);
  const [isQueueOpen, setIsQueueOpen] = useState(false);
  const [previousVolume, setPreviousVolume] = useState(1);

  // Keyboard shortcuts for player-specific actions
  useKeyboardShortcuts({
    onToggleFullscreen: () => setIsFullScreen(prev => !prev),
    onToggleQueue: () => setIsQueueOpen(prev => !prev),
  });

  // Sync Play/Pause
  useEffect(() => {
    if (audioRef.current) {
      if (isPlaying) audioRef.current.play().catch(() => setIsPlaying(false));
      else audioRef.current.pause();
    }
  }, [isPlaying, currentSong]);

  // Sync Volume
  useEffect(() => {
    if (audioRef.current) audioRef.current.volume = volume;
  }, [volume]);

  const handleTimeUpdate = () => {
    if (audioRef.current) {
      setCurrentTime(audioRef.current.currentTime);
      setDuration(audioRef.current.duration || 0);
    }
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const time = Number(e.target.value);
    if (audioRef.current) {
      audioRef.current.currentTime = time;
      setCurrentTime(time);
    }
  };

  const handleMuteToggle = () => {
    if (volume > 0) {
      setPreviousVolume(volume);
      setVolume(0);
    } else {
      setVolume(previousVolume || 1);
    }
  };

  if (!currentSong) return null;

  return (
    <>
      <div
        className="fixed bottom-0 left-0 right-0 h-24 glass z-40 px-4 md:px-8 flex items-center justify-between transition-all border-t border-gray-200 dark:border-white/10"
        onClick={(e) => {
          if ((e.target as HTMLElement).tagName !== 'BUTTON' && (e.target as HTMLElement).tagName !== 'INPUT') {
            setIsFullScreen(true);
          }
        }}
      >
        {/* Audio Engine */}
        <audio
          ref={audioRef}
          src={getStreamUrl(currentSong.id)}
          loop={repeatMode === 'one'}
          onTimeUpdate={handleTimeUpdate}
          onEnded={() => {
            if (repeatMode !== 'one') playNext();
          }}
          onLoadedMetadata={(e) => setDuration(e.currentTarget.duration)}
        />

        {/* LEFT: Art & Text */}
        <div className="flex items-center gap-4 w-1/3 min-w-0 cursor-pointer group">
          <div className="w-14 h-14 rounded-md shadow-lg overflow-hidden flex-shrink-0 border border-white/10 bg-gray-800 relative">
            {currentSong.album_id ? (
              <img src={getCoverUrl(currentSong.album_id)} alt="art" className="w-full h-full object-cover" />
            ) : <Disc className="m-auto text-gray-500" />}
            <div className="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition">
              <ChevronUp size={20} className="text-white" />
            </div>
          </div>
          <div className="truncate hidden sm:block">
            <h3 className="font-semibold text-apple-text truncate text-sm">{currentSong.title}</h3>
            <p className="text-xs text-apple-subtext truncate">{currentSong.artist}</p>
          </div>
        </div>

        {/* CENTER: Controls & Progress */}
        <div className="flex flex-col items-center w-1/3 max-w-lg gap-1">
          <div className="flex items-center gap-6">
            {/* Shuffle */}
            <button
              onClick={(e) => { e.stopPropagation(); toggleShuffle(); }}
              className={cn("transition hover:scale-110 hidden sm:block", isShuffle ? "text-apple-accent" : "text-apple-subtext hover:text-apple-text")}
              title="Shuffle"
            >
              <Shuffle size={18} />
            </button>

            <button onClick={(e) => { e.stopPropagation(); playPrev(); }} className="text-apple-text hover:text-apple-accent transition" title="Previous (←)">
              <SkipBack size={24} />
            </button>

            <button onClick={(e) => { e.stopPropagation(); togglePlay(); }} className="w-10 h-10 bg-apple-text text-white rounded-full flex items-center justify-center hover:scale-105 shadow-md transition-transform" title="Play/Pause (Space)">
              {isPlaying ? <Pause size={20} fill="currentColor" /> : <Play size={20} fill="currentColor" className="ml-1" />}
            </button>

            <button onClick={(e) => { e.stopPropagation(); playNext(); }} className="text-apple-text hover:text-apple-accent transition" title="Next (→)">
              <SkipForward size={24} />
            </button>

            {/* Repeat */}
            <button
              onClick={(e) => { e.stopPropagation(); toggleRepeat(); }}
              className={cn("transition hover:scale-110 hidden sm:block relative", repeatMode !== 'off' ? "text-apple-accent" : "text-apple-subtext hover:text-apple-text")}
              title={repeatMode === 'off' ? 'Repeat Off' : repeatMode === 'one' ? 'Repeat One' : 'Repeat All'}
            >
              {repeatMode === 'one' ? <Repeat1 size={18} /> : <Repeat size={18} />}
              {repeatMode !== 'off' && <div className="absolute -bottom-2 left-1/2 -translate-x-1/2 w-1 h-1 bg-current rounded-full" />}
            </button>
          </div>

          {/* Progress Bar */}
          <div className="w-full flex items-center gap-3 text-[10px] text-apple-subtext font-medium">
            <span className="min-w-[30px] text-right">{formatTime(currentTime)}</span>
            <div className="flex-1 h-1 bg-gray-300 dark:bg-white/10 rounded-full relative group">
              <div
                className="absolute top-0 left-0 h-full bg-apple-text rounded-full transition-all"
                style={{ width: `${(currentTime / duration) * 100}%` }}
              />
              <input
                type="range" min={0} max={duration || 100} value={currentTime}
                onClick={(e) => e.stopPropagation()}
                onChange={handleSeek}
                className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                title="Seek (Shift + ←/→)"
              />
            </div>
            <span className="min-w-[30px]">{formatTime(duration)}</span>
          </div>
        </div>

        {/* RIGHT: Volume & Queue */}
        <div className="flex items-center justify-end gap-3 w-1/3">
          <div className="hidden sm:flex items-center gap-2 w-32 group relative">
            <button
              onClick={(e) => { e.stopPropagation(); handleMuteToggle(); }}
              className="text-apple-subtext hover:text-apple-text transition"
              title={volume > 0 ? "Mute (M)" : "Unmute (M)"}
            >
              {volume === 0 ? <VolumeX size={16} /> : <Volume2 size={16} />}
            </button>
            <div className="relative flex-1">
              <input
                type="range" min={0} max={1} step={0.01} value={volume}
                onClick={(e) => e.stopPropagation()}
                onChange={(e) => setVolume(parseFloat(e.target.value))}
                className="w-full h-1 bg-gray-300 dark:bg-white/20 rounded-lg appearance-none cursor-pointer accent-apple-text hover:accent-apple-accent transition"
                title="Volume (↑/↓)"
              />
              {/* Volume percentage tooltip */}
              <div className="absolute -top-8 left-1/2 -translate-x-1/2 bg-gray-900 text-white text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition pointer-events-none whitespace-nowrap z-50">
                {Math.round(volume * 100)}%
              </div>
            </div>
          </div>
          <button
            onClick={(e) => { e.stopPropagation(); setIsQueueOpen(true); }}
            className="p-2 hover:bg-gray-100 dark:hover:bg-white/10 rounded-full transition text-apple-subtext hover:text-apple-text"
            title="View Queue (Q)"
          >
            <ListMusic size={20} />
          </button>
        </div>
      </div>

      <FullScreenPlayer isOpen={isFullScreen} onClose={() => setIsFullScreen(false)} />
      <QueuePanel isOpen={isQueueOpen} onClose={() => setIsQueueOpen(false)} />
    </>
  );
}