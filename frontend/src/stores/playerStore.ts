import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { Song } from '../types';
import { getSong } from '../lib/api';

type RepeatMode = 'off' | 'all' | 'one';

interface PlayerState {
  isPlaying: boolean;
  currentSong: Song | null;
  volume: number;
  queue: Song[];
  originalQueue: Song[];
  currentIndex: number;
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
      currentIndex: -1,
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
          // Fisher-Yates Shuffle
          for (let i = newQueue.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [newQueue[i], newQueue[j]] = [newQueue[j], newQueue[i]];
          }
          // Keep current song first
          if (currentSong) {
            const idx = newQueue.findIndex(s => s.id === currentSong.id);
            if (idx > -1) {
              newQueue.splice(idx, 1);
              newQueue.unshift(currentSong);
            }
          }
          set({ isShuffle: true, queue: newQueue });
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
        if (!currentSong) return;

        if (repeatMode === 'one') {
          set({ isPlaying: true });
          const audio = document.querySelector('audio');
          if (audio) { audio.currentTime = 0; audio.play(); }
          return;
        }

        const idx = queue.findIndex(s => s.id === currentSong.id);
        if (idx < queue.length - 1) {
          set({ currentSong: queue[idx + 1], isPlaying: true });
        } else if (repeatMode === 'all') {
          set({ currentSong: queue[0], isPlaying: true });
        } else {
          set({ isPlaying: false });
        }
      },

      playPrev: () => {
        const { queue, currentSong } = get();
        if (!currentSong) return;
        const idx = queue.findIndex(s => s.id === currentSong.id);
        if (idx > 0) {
          set({ currentSong: queue[idx - 1], isPlaying: true });
        } else {
          // If at start of list, restart song
          const audio = document.querySelector('audio');
          if (audio) audio.currentTime = 0;
        }
      },

      // Queue management actions
      reorderQueue: (oldIndex: number, newIndex: number) => {
        const { queue, currentSong } = get();
        const newQueue = [...queue];
        const [removed] = newQueue.splice(oldIndex, 1);
        newQueue.splice(newIndex, 0, removed);

        // Update current index if needed
        const newCurrentIndex = newQueue.findIndex(s => s.id === currentSong?.id);

        set({ queue: newQueue, currentIndex: newCurrentIndex });
      },

      removeFromQueue: (index: number) => {
        const { queue, currentSong } = get();
        const newQueue = queue.filter((_, i) => i !== index);

        // Update current index if needed
        const newCurrentIndex = newQueue.findIndex(s => s.id === currentSong?.id);

        set({ queue: newQueue, currentIndex: newCurrentIndex });
      },

      addToQueue: (song: Song) => {
        const { queue, originalQueue, currentSong } = get();
        const currentIdx = queue.findIndex(s => s.id === currentSong?.id);

        // Insert after current song, or at end if no current song
        const newQueue = [...queue];
        const newOriginalQueue = [...originalQueue];
        if (currentIdx >= 0) {
          newQueue.splice(currentIdx + 1, 0, song);
        } else {
          newQueue.push(song);
        }
        // Also add to originalQueue to keep them in sync
        newOriginalQueue.push(song);

        set({ queue: newQueue, originalQueue: newOriginalQueue });
      },

      clearQueue: () => {
        set({ queue: [], originalQueue: [], currentIndex: -1, currentSong: null, isPlaying: false });
      },

      validateState: async () => {
        const { currentSong } = get();
        if (!currentSong) return;

        try {
          // Check if the current song still exists in the backend
          await getSong(currentSong.id);
        } catch (error) {
          console.warn('Current song not found in backend (likely db reset). Clearing player state.');
          set({
            queue: [],
            originalQueue: [],
            currentIndex: -1,
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