// Synced lyrics component with virtualization for minimal RAM usage
import { useEffect, useRef, useState, useMemo } from 'react';
import { Virtuoso, VirtuosoHandle } from 'react-virtuoso';
import { cn } from '../../lib/utils';

interface LyricLine {
    time: number;
    text: string;
}

interface SyncedLyricsProps {
    lyrics: string;
    currentTime: number;
    className?: string;
}

// Parse LRC format lyrics
function parseLRC(lrc: string): LyricLine[] {
    const lines: LyricLine[] = [];
    const lrcLines = lrc.split('\n');

    for (const line of lrcLines) {
        // Match [mm:ss.xx] or [mm:ss] format
        const match = line.match(/\[(\d{2}):(\d{2})(?:\.(\d{2,3}))?\](.*)/);
        if (match) {
            const minutes = parseInt(match[1]);
            const seconds = parseInt(match[2]);
            const milliseconds = match[3] ? parseInt(match[3].padEnd(3, '0')) : 0;
            const text = match[4].trim();

            const time = minutes * 60 + seconds + milliseconds / 1000;
            lines.push({ time, text });
        }
    }

    return lines.sort((a, b) => a.time - b.time);
}

export function SyncedLyrics({ lyrics, currentTime, className }: SyncedLyricsProps) {
    const [currentLineIndex, setCurrentLineIndex] = useState(-1);
    const [isReady, setIsReady] = useState(false);
    const virtuosoRef = useRef<VirtuosoHandle>(null);

    // Delay rendering slightly to allow parent container height to settle (fixes "bugged" first load)
    useEffect(() => {
        const timer = setTimeout(() => setIsReady(true), 100);
        return () => clearTimeout(timer);
    }, []);

    // Memoize parsing so it only runs when raw lyrics change
    const parsedLines = useMemo(() => parseLRC(lyrics), [lyrics]);

    // Reset index when song (lyrics) changes
    useEffect(() => {
        setCurrentLineIndex(-1);
    }, [lyrics]);

    // Calculate active index logic (Same efficient search as before)
    useEffect(() => {
        if (parsedLines.length === 0) return;

        // Add 0.3s offset for better sync feel
        const adjustedTime = currentTime + 0.3;

        // Efficient search strategy
        let nextIndex = -1;

        const nextPossible = currentLineIndex + 1;
        if (nextPossible < parsedLines.length && parsedLines[nextPossible].time <= adjustedTime) {
            for (let i = nextPossible; i < parsedLines.length; i++) {
                if (parsedLines[i].time <= adjustedTime) {
                    nextIndex = i;
                } else {
                    break;
                }
            }
        } else if (currentLineIndex >= 0 && parsedLines[currentLineIndex].time <= adjustedTime) {
            nextIndex = currentLineIndex;
        } else {
            for (let i = 0; i < parsedLines.length; i++) {
                if (parsedLines[i].time <= adjustedTime) {
                    nextIndex = i;
                } else {
                    break;
                }
            }
        }

        if (nextIndex !== currentLineIndex) {
            setCurrentLineIndex(nextIndex);

            // Sync scroll with virtualization
            if (nextIndex !== -1 && virtuosoRef.current) {
                virtuosoRef.current.scrollToIndex({
                    index: nextIndex,
                    align: 'center',
                    behavior: 'smooth'
                });
            }
        }
    }, [currentTime, parsedLines, currentLineIndex]);

    if (parsedLines.length === 0) {
        return (
            <div className={cn("whitespace-pre-wrap text-2xl font-bold text-white/80 leading-relaxed space-y-8 pb-20 px-4", className)}>
                {lyrics}
            </div>
        );
    }

    return (
        <div
            className={cn("h-full", className)}
            style={{
                maskImage: 'linear-gradient(to bottom, transparent 0%, black 10%, black 90%, transparent 100%)',
                WebkitMaskImage: 'linear-gradient(to bottom, transparent 0%, black 10%, black 90%, transparent 100%)'
            }}
        >
            {isReady ? (
                <Virtuoso
                    ref={virtuosoRef}
                    className="no-scrollbar"
                    style={{ height: '100%' }}
                    totalCount={parsedLines.length}
                    data={parsedLines}
                    // Header provides the top spacing to center the first line
                    components={{
                        Header: () => <div style={{ height: '40vh' }} />,
                        Footer: () => <div style={{ height: '40vh' }} />,
                    }}
                    itemContent={(index, line) => (
                        <div
                            className={cn(
                                "py-3 px-4 transition-all duration-300 leading-relaxed cursor-pointer origin-left will-change-transform backface-visibility-hidden",
                                index === currentLineIndex
                                    ? "text-2xl font-bold text-white scale-100 opacity-100"
                                    : "text-2xl font-medium text-white scale-90 opacity-40 blur-[0.5px]"
                            )}
                            style={{ transform: index !== currentLineIndex ? 'scale(0.9)' : 'scale(1)', transformOrigin: 'left center' }}
                            onClick={() => {
                                // Optional: Click to seek could go here
                                const audio = document.querySelector('audio');
                                if (audio) {
                                    audio.currentTime = line.time;
                                }
                            }}
                        >
                            {line.text || '♪'}
                        </div>
                    )}
                />
            ) : (
                <div className="h-full flex items-center justify-center opacity-0 transition-opacity duration-300">
                    {/* Placeholder waiting for layout */}
                </div>
            )}
        </div>
    );
}
