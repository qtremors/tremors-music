import { useEffect, useState } from 'react';
import { usePlayerStore } from '../stores/playerStore';
import { getCoverUrl, getLyrics } from '../lib/api';
import { cn, formatTime } from '../lib/utils';
import { ChevronDown, Play, Pause, SkipBack, SkipForward, MessageSquare, Volume2, Repeat, Shuffle, Repeat1 } from 'lucide-react';
import { AnimatePresence, motion } from 'framer-motion';

interface FullScreenPlayerProps {
  isOpen: boolean;
  onClose: () => void;
}

export function FullScreenPlayer({ isOpen, onClose }: FullScreenPlayerProps) {
  const { 
    currentSong, isPlaying, togglePlay, playNext, playPrev, 
    volume, setVolume, repeatMode, toggleRepeat, isShuffle, toggleShuffle 
  } = usePlayerStore();
  
  const [lyrics, setLyrics] = useState<{ plainLyrics?: string, syncedLyrics?: string } | null>(null);
  const [showLyrics, setShowLyrics] = useState(false); 
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);

  useEffect(() => {
    if (isOpen && currentSong) {
      setLyrics(null);
      getLyrics(currentSong.id).then(setLyrics).catch(() => setLyrics(null));
    }
  }, [currentSong?.id, isOpen]);

  useEffect(() => {
    let interval: any;
    if (isOpen) {
      interval = setInterval(() => {
        const audio = document.querySelector('audio');
        if (audio) {
          setCurrentTime(audio.currentTime);
          setDuration(audio.duration || 0);
        }
      }, 200);
    }
    return () => clearInterval(interval);
  }, [isOpen]);

  const handleSeek = (e: any) => {
    const val = Number(e.target.value);
    const audio = document.querySelector('audio');
    if (audio) {
        audio.currentTime = val;
        setCurrentTime(val);
    }
  };

  if (!currentSong) return null;

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ y: '100%' }}
          animate={{ y: 0 }}
          exit={{ y: '100%' }}
          transition={{ type: 'spring', damping: 25, stiffness: 200 }}
          className="fixed inset-0 z-[100] bg-[#1c1c1e] text-white flex flex-col overflow-hidden"
        >
          {/* --- Background --- */}
          <div className="absolute inset-0 bg-cover bg-center opacity-30 blur-[100px] scale-125 pointer-events-none transform-gpu"
             style={{ backgroundImage: `url(${getCoverUrl(currentSong.album_id!)})` }}
          />
          <div className="absolute inset-0 bg-black/40" />

          {/* --- Header --- */}
          <div className="relative z-20 flex justify-between items-center px-8 py-6 flex-shrink-0">
             <button onClick={onClose} className="p-2 bg-white/10 rounded-full hover:bg-white/20"><ChevronDown size={20} /></button>
             <div className="w-10" />
          </div>

          {/* --- Main Content --- */}
          <div className="relative z-10 flex-1 min-h-0 w-full max-w-screen-2xl mx-auto px-8 pb-8 flex gap-12 items-center">
            
            {/* LEFT COLUMN: Art & Controls */}
            <div className={cn(
                "flex flex-col justify-center h-full transition-all duration-500 ease-in-out mx-auto",
                showLyrics ? "w-1/2 max-w-xl" : "w-full max-w-lg"
            )}>
               
               {/* Flexible Art Container */}
               <div className="flex-1 min-h-0 flex items-center justify-center mb-6 w-full">
                  <div className="aspect-square h-full max-h-[50vh] w-auto rounded-xl shadow-2xl overflow-hidden border border-white/10 bg-gray-800 relative flex-shrink-0">
                    <img src={getCoverUrl(currentSong.album_id!)} className="w-full h-full object-cover" />
                  </div>
               </div>

               {/* Metadata & Controls */}
               <div className="flex-shrink-0 space-y-6 w-full">
                   <div className={showLyrics ? "text-left" : "text-center"}>
                      <h2 className="text-2xl font-bold truncate leading-tight">{currentSong.title}</h2>
                      <p className="text-lg text-white/60 truncate mt-1">{currentSong.artist}</p>
                      
                      {/* TECH BADGES */}
                      <div className={cn("flex gap-2 mt-3 opacity-60", showLyrics ? "justify-start" : "justify-center")}>
                         <span className="px-1.5 py-0.5 rounded border border-white/30 text-[9px] font-bold uppercase tracking-widest text-white/80">
                            {currentSong.format?.toUpperCase() || 'MP3'}
                         </span>
                         {currentSong.bitrate && (
                            <span className="px-1.5 py-0.5 rounded border border-white/30 text-[9px] font-bold uppercase tracking-widest text-white/80">
                                {currentSong.bitrate} kbps
                            </span>
                         )}
                         {currentSong.sample_rate && (
                            <span className="px-1.5 py-0.5 rounded border border-white/30 text-[9px] font-bold uppercase tracking-widest text-white/80">
                                {(currentSong.sample_rate / 1000).toFixed(1)} kHz
                            </span>
                         )}
                      </div>
                   </div>

                   {/* Progress */}
                   <div className="space-y-2">
                      <div className="relative w-full h-1 bg-white/20 rounded-full group cursor-pointer">
                         <div 
                           className="absolute top-0 left-0 h-full bg-white rounded-full" 
                           style={{ width: `${(currentTime / duration) * 100}%` }}
                         />
                         <input 
                           type="range" min={0} max={duration || 100} value={currentTime} 
                           onChange={handleSeek}
                           className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                         />
                      </div>
                      <div className="flex justify-between text-xs font-medium text-white/50">
                         <span>{formatTime(currentTime)}</span>
                         <span>{formatTime(duration)}</span>
                      </div>
                   </div>

                   {/* Controls */}
                   <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2 w-24">
                          <Volume2 size={16} className="text-white/60" />
                          <input 
                            type="range" min={0} max={1} step={0.01} value={volume} 
                            onChange={e => setVolume(parseFloat(e.target.value))}
                            className="w-16 h-1 bg-white/20 rounded-lg appearance-none cursor-pointer accent-white"
                          />
                      </div>

                      <div className="flex items-center gap-8">
                         <button onClick={playPrev} className="hover:text-white/80 transition hover:scale-105"><SkipBack size={28} fill="currentColor" /></button>
                         <button onClick={togglePlay} className="hover:scale-105 transition">
                           {isPlaying ? <Pause size={48} fill="currentColor" /> : <Play size={48} fill="currentColor" />}
                         </button>
                         <button onClick={playNext} className="hover:text-white/80 transition hover:scale-105"><SkipForward size={28} fill="currentColor" /></button>
                      </div>

                      <div className="flex items-center gap-3 w-24 justify-end">
                          <button onClick={toggleShuffle} className={cn("transition", isShuffle ? "text-apple-accent" : "text-white/40 hover:text-white")}>
                              <Shuffle size={18} />
                          </button>
                          <button onClick={toggleRepeat} className={cn("transition relative", repeatMode !== 'off' ? "text-apple-accent" : "text-white/40 hover:text-white")}>
                              {repeatMode === 'one' ? <Repeat1 size={18} /> : <Repeat size={18} />}
                              {repeatMode !== 'off' && <div className="absolute -bottom-2 left-1/2 -translate-x-1/2 w-0.5 h-0.5 bg-current rounded-full" />}
                          </button>
                          <button 
                            onClick={() => setShowLyrics(!showLyrics)} 
                            className={cn("transition p-1.5 rounded-md", showLyrics ? "bg-white/20 text-white" : "text-white/40 hover:text-white")}
                          >
                              <MessageSquare size={18} fill={showLyrics ? "currentColor" : "none"} />
                          </button>
                      </div>
                   </div>
               </div>
            </div>

            {/* RIGHT COLUMN: Lyrics */}
            {showLyrics && (
              <div className="hidden md:flex flex-col w-1/2 h-full animate-in fade-in slide-in-from-right-10 duration-300">
                 <h3 className="text-xs font-bold text-white/40 uppercase tracking-widest mb-6 flex-shrink-0">Lyrics</h3>
                 <div className="flex-1 overflow-y-auto no-scrollbar mask-image-linear-gradient">
                    {lyrics?.syncedLyrics || lyrics?.plainLyrics ? (
                        <div className="whitespace-pre-wrap text-2xl font-bold text-white/80 leading-relaxed space-y-8 pb-20">
                            {lyrics.syncedLyrics || lyrics.plainLyrics}
                        </div>
                    ) : (
                        <div className="flex items-center justify-center h-full text-white/30 italic text-xl font-medium">
                            Searching for lyrics...
                        </div>
                    )}
                 </div>
              </div>
            )}

          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}