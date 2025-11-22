import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface ThemeState {
  theme: 'light' | 'dark';
  accentColor: string;
  showSongListArt: boolean;
  
  setTheme: (theme: 'light' | 'dark') => void;
  setAccentColor: (rgb: string) => void;
  setShowSongListArt: (show: boolean) => void;
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set) => ({
      theme: 'light',
      accentColor: '250 45 72',
      showSongListArt: true, // Default to true

      setTheme: (theme) => {
        set({ theme });
        if (theme === 'dark') document.documentElement.classList.add('dark');
        else document.documentElement.classList.remove('dark');
      },
      setAccentColor: (rgb) => {
        set({ accentColor: rgb });
        document.documentElement.style.setProperty('--accent-color', rgb);
      },
      setShowSongListArt: (show) => set({ showSongListArt: show }),
    }),
    {
      name: 'theme-storage',
      onRehydrateStorage: () => (state) => {
        if (state?.theme === 'dark') document.documentElement.classList.add('dark');
        if (state?.accentColor) document.documentElement.style.setProperty('--accent-color', state.accentColor);
      }
    }
  )
);