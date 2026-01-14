import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { Song } from '../types';
import { getSong } from '../lib/api';
import { shuffleArray } from '../lib/utils';

type RepeatMode = 'off' | 'all' | 'one';

interface PlayerState {
  isPlaying: boolean;
  currentSong: Song | null;
  volume: number;
  queue: Song[];
  originalQueue: Song[];
  repeatMode: RepeatMode;
  isShuffle: boolean;

  playSong: (song: Song) => void;
  togglePlay: () => void;
  setIsPlaying: (isPlaying: boolean) => void;
  setVolume: (val: number) => void;
  setQueue: (songs: Song[]) => void;
  playNext: () => void;
  playPrev: () => void;
  toggleShuffle: () => void;
  toggleRepeat: () => void;
  reorderQueue: (oldIndex: number, newIndex: number) => void;
  removeFromQueue: (index: number) => void;
  addToQueue: (song: Song) => void;
  clearQueue: () => void;
  validateState: () => Promise<void>;
}

export const usePlayerStore = create<PlayerState>()(
  persist(
    (set, get) => ({
      isPlaying: false,
      currentSong: null,
      volume: 1,
      queue: [],
      originalQueue: [],
      repeatMode: 'off',
      isShuffle: false,

      playSong: (song) => set({ currentSong: song, isPlaying: true }),

      togglePlay: () => set((state) => ({ isPlaying: !state.isPlaying })),

      setIsPlaying: (isPlaying) => set({ isPlaying }),

      setVolume: (val) => set({ volume: val }),

      setQueue: (songs) => set({ queue: songs, originalQueue: songs }),

      toggleShuffle: () => {
        const { isShuffle, originalQueue, currentSong } = get();
        if (!isShuffle) {
          const newQueue = [...originalQueue];
          // Use shared Fisher-Yates shuffle
          const shuffled = shuffleArray(newQueue);

          // Keep current song first logic...
          const queueToSet = shuffled;
          if (currentSong) {
            const idx = queueToSet.findIndex(s => s.id === currentSong.id);
            if (idx > -1) {
              queueToSet.splice(idx, 1);
              queueToSet.unshift(currentSong);
            }
          }
          set({ isShuffle: true, queue: queueToSet });
        } else {
          set({ isShuffle: false, queue: originalQueue });
        }
      },

      toggleRepeat: () => {
        const modes: RepeatMode[] = ['off', 'all', 'one'];
        const next = modes[(modes.indexOf(get().repeatMode) + 1) % modes.length];
        set({ repeatMode: next });
      },

      playNext: () => {
        const { queue, currentSong, repeatMode } = get();
        if (queue.length === 0) return;

        // Handle 'one' repeat mode first
        if (repeatMode === 'one' && currentSong) {
          set({ isPlaying: true });
          const audio = document.querySelector('audio');
          if (audio) { audio.currentTime = 0; audio.play(); }
          return;
        }

        const currentIndex = currentSong ? queue.findIndex(s => s.id === currentSong.id) : -1;

        let nextIndex = currentIndex + 1;
        if (nextIndex >= queue.length) {
          if (repeatMode === 'all') {
            nextIndex = 0; // Loop back to the beginning
          } else {
            set({ isPlaying: false }); // End of queue, stop playing
            return;
          }
        }

        set({ currentSong: queue[nextIndex], isPlaying: true });
      },

      playPrev: () => {
        const { queue, currentSong, repeatMode } = get();
        if (queue.length === 0) return;

        const currentIndex = currentSong ? queue.findIndex(s => s.id === currentSong.id) : -1;

        let prevIndex = currentIndex - 1;
        if (prevIndex < 0) {
          if (repeatMode === 'all') {
            prevIndex = queue.length - 1;
          } else {
            return; // Start of queue
          }
        }

        set({ currentSong: queue[prevIndex], isPlaying: true });
      },

      // Queue management actions
      reorderQueue: (oldIndex: number, newIndex: number) => {
        set((state) => {
          const newQueue = [...state.queue];
          const [moved] = newQueue.splice(oldIndex, 1);
          newQueue.splice(newIndex, 0, moved);

          return { queue: newQueue };
        });
      },

      removeFromQueue: (index: number) => {
        const { queue } = get();
        const newQueue = [...queue];
        newQueue.splice(index, 1);
        set({ queue: newQueue });
      },

      addToQueue: (song: Song) => {
        const { queue, originalQueue, currentSong } = get();
        const currentIdx = queue.findIndex(s => s.id === currentSong?.id);

        const newQueue = [...queue];
        const newOriginalQueue = [...originalQueue];
        if (currentIdx >= 0) {
          newQueue.splice(currentIdx + 1, 0, song);
        } else {
          newQueue.push(song);
        }

        // Also properly insert into originalQueue relative to current song
        // Find current song in originalQueue
        const originalCurrentIdx = newOriginalQueue.findIndex(s => s.id === currentSong?.id);
        if (originalCurrentIdx >= 0) {
          newOriginalQueue.splice(originalCurrentIdx + 1, 0, song);
        } else {
          newOriginalQueue.push(song);
        }

        set({ queue: newQueue, originalQueue: newOriginalQueue });
      },

      clearQueue: () => {
        set({ queue: [], originalQueue: [], currentSong: null, isPlaying: false });
      },

      validateState: async () => {
        const { currentSong } = get();
        if (!currentSong) return;

        try {
          // Check if the current song still exists in the backend
          await getSong(currentSong.id);
        } catch {
          console.warn('Current song not found in backend (likely db reset). Clearing player state.');
          set({
            queue: [],
            originalQueue: [],
            currentSong: null,
            isPlaying: false
          });
        }
      },
    }),
    {
      name: 'tremors-player-storage',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        currentSong: state.currentSong,
        volume: state.volume,
        queue: state.queue,
        originalQueue: state.originalQueue,
        repeatMode: state.repeatMode,
        isShuffle: state.isShuffle,
      }),
    }
  )
);