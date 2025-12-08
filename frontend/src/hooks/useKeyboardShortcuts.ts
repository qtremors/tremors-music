import { useEffect } from 'react';
import { usePlayerStore } from '../stores/playerStore';

interface KeyboardShortcutsOptions {
    onToggleFullscreen?: () => void;
    onToggleLyrics?: () => void;
    onToggleQueue?: () => void;
    onFocusSearch?: () => void;
}

/**
 * Keyboard shortcuts for the music player.
 * 
 * Shortcuts are disabled when typing in input fields.
 * Uses Ctrl/Cmd modifiers for letter shortcuts to avoid conflicts.
 * 
 * Available shortcuts:
 * - Space: Play/Pause
 * - Arrow Right: Next song
 * - Arrow Left: Previous song
 * - Shift + Arrow Right: Seek forward 10s
 * - Shift + Arrow Left: Seek backward 10s
 * - Arrow Up: Volume up
 * - Arrow Down: Volume down
 * - Ctrl/Cmd + M: Mute/Unmute
 * - Ctrl/Cmd + F: Toggle fullscreen player
 * - Ctrl/Cmd + L: Toggle lyrics
 * - Ctrl/Cmd + Q: Toggle queue
 * - Ctrl/Cmd + K or /: Focus search (works from anywhere)
 * - Escape: Blur search / close modals
 */
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
            const target = e.target as HTMLElement;
            const isInInput = (
                target.tagName === 'INPUT' ||
                target.tagName === 'TEXTAREA' ||
                target.isContentEditable
            );

            const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
            const modifierKey = isMac ? e.metaKey : e.ctrlKey;

            // === GLOBAL SHORTCUTS (work even in input fields) ===

            // Escape - blur input / close things
            if (e.key === 'Escape') {
                if (isInInput) {
                    (target as HTMLInputElement).blur();
                }
                return;
            }

            // Ctrl/Cmd + K or "/" - Focus search (global, works anywhere)
            if ((modifierKey && e.key.toLowerCase() === 'k') || (e.key === '/' && !isInInput)) {
                e.preventDefault();
                options.onFocusSearch?.();
                return;
            }

            // === SHORTCUTS DISABLED IN INPUT FIELDS ===
            if (isInInput) {
                return; // Don't process any other shortcuts when typing
            }

            // === PLAYER SHORTCUTS (only when not typing) ===

            // Space - Play/Pause
            if (e.key === ' ') {
                e.preventDefault();
                if (currentSong) togglePlay();
                return;
            }

            // Arrow keys for navigation and seeking
            if (e.key === 'ArrowRight') {
                e.preventDefault();
                if (e.shiftKey) {
                    // Shift + Right - Seek forward 10s
                    const audio = document.querySelector('audio');
                    if (audio) audio.currentTime += 10;
                } else {
                    // Right - Next song
                    playNext();
                }
                return;
            }

            if (e.key === 'ArrowLeft') {
                e.preventDefault();
                if (e.shiftKey) {
                    // Shift + Left - Seek backward 10s
                    const audio = document.querySelector('audio');
                    if (audio) audio.currentTime -= 10;
                } else {
                    // Left - Previous song
                    playPrev();
                }
                return;
            }

            if (e.key === 'ArrowUp') {
                e.preventDefault();
                setVolume(Math.min(1, volume + 0.1));
                return;
            }

            if (e.key === 'ArrowDown') {
                e.preventDefault();
                setVolume(Math.max(0, volume - 0.1));
                return;
            }

            // === MODIFIER KEY SHORTCUTS ===
            // These require Ctrl/Cmd to avoid conflicts with typing

            if (modifierKey) {
                switch (e.key.toLowerCase()) {
                    case 'm': // Ctrl/Cmd + M - Mute/Unmute
                        e.preventDefault();
                        setVolume(volume > 0 ? 0 : 1);
                        break;

                    case 'f': // Ctrl/Cmd + F - Toggle fullscreen (if not browser's find)
                        // Note: Some browsers reserve Ctrl+F for find
                        // Only prevent default if we have a fullscreen handler
                        if (options.onToggleFullscreen) {
                            e.preventDefault();
                            options.onToggleFullscreen();
                        }
                        break;

                    case 'l': // Ctrl/Cmd + L - Toggle lyrics
                        e.preventDefault();
                        options.onToggleLyrics?.();
                        break;

                    case 'q': // Ctrl/Cmd + Q - Toggle queue
                        // Note: Cmd+Q quits on Mac, so only on Windows/Linux
                        if (!isMac) {
                            e.preventDefault();
                            options.onToggleQueue?.();
                        }
                        break;
                }
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
