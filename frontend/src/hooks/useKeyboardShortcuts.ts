import { useEffect } from 'react';
import { usePlayerStore } from '../stores/playerStore';

interface KeyboardShortcutsOptions {
    onToggleFullscreen?: () => void;
    onToggleLyrics?: () => void;
    onToggleQueue?: () => void;
    onFocusSearch?: () => void;
}

export function useKeyboardShortcuts(options: KeyboardShortcutsOptions = {}) {
    const {
        togglePlay,
        playNext,
        playPrev,
        setVolume,
        volume,
        currentSong
    } = usePlayerStore();

    useEffect(() => {
        const handleKeyPress = (e: KeyboardEvent) => {
            // Don't trigger shortcuts when typing in inputs
            const target = e.target as HTMLElement;
            if (
                target.tagName === 'INPUT' ||
                target.tagName === 'TEXTAREA' ||
                target.isContentEditable
            ) {
                // Allow search shortcuts even in inputs
                if (e.key !== '/' && e.key !== 's' && e.key !== 'S') {
                    return;
                }
            }

            // Prevent default for shortcuts we handle
            const shouldPreventDefault = [
                ' ', 'ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown',
                'm', 'M', 'f', 'F', 'l', 'L', 'q', 'Q', 's', 'S', '/'
            ].includes(e.key);

            if (shouldPreventDefault) {
                e.preventDefault();
            }

            // Handle shortcuts
            switch (e.key) {
                case ' ': // Space - Play/Pause
                    if (currentSong) togglePlay();
                    break;

                case 'ArrowRight': // Right Arrow
                    if (e.shiftKey) {
                        // Shift + Right - Seek forward 10s
                        const audio = document.querySelector('audio');
                        if (audio) audio.currentTime += 10;
                    } else {
                        // Right - Next song
                        playNext();
                    }
                    break;

                case 'ArrowLeft': // Left Arrow
                    if (e.shiftKey) {
                        // Shift + Left - Seek backward 10s
                        const audio = document.querySelector('audio');
                        if (audio) audio.currentTime -= 10;
                    } else {
                        // Left - Previous song
                        playPrev();
                    }
                    break;

                case 'ArrowUp': // Up Arrow - Volume up
                    setVolume(Math.min(1, volume + 0.1));
                    break;

                case 'ArrowDown': // Down Arrow - Volume down
                    setVolume(Math.max(0, volume - 0.1));
                    break;

                case 'm':
                case 'M': // M - Mute/Unmute
                    setVolume(volume > 0 ? 0 : 1);
                    break;

                case 'f':
                case 'F': // F - Toggle fullscreen player
                    options.onToggleFullscreen?.();
                    break;

                case 'l':
                case 'L': // L - Toggle lyrics
                    options.onToggleLyrics?.();
                    break;

                case 'q':
                case 'Q': // Q - Toggle queue
                    options.onToggleQueue?.();
                    break;

                case 's':
                case 'S':
                case '/': // S or / - Focus search
                    options.onFocusSearch?.();
                    break;
            }
        };

        window.addEventListener('keydown', handleKeyPress);
        return () => window.removeEventListener('keydown', handleKeyPress);
    }, [
        togglePlay,
        playNext,
        playPrev,
        setVolume,
        volume,
        currentSong,
        options
    ]);
}
