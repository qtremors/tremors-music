// Synced lyrics component with auto-scroll
import { useEffect, useRef, useState } from 'react';
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
    const [parsedLines, setParsedLines] = useState<LyricLine[]>([]);
    const [currentLineIndex, setCurrentLineIndex] = useState(-1);
    const containerRef = useRef<HTMLDivElement>(null);
    const activeLineRef = useRef<HTMLDivElement>(null);

    // Parse lyrics on mount or when lyrics change
    useEffect(() => {
        const lines = parseLRC(lyrics);
        setParsedLines(lines);
    }, [lyrics]);

    // Update current line based on time with offset to sync better
    useEffect(() => {
        if (parsedLines.length === 0) return;

        // Add 0.3s offset to account for processing delay and make it feel more responsive
        const adjustedTime = currentTime + 0.3;

        // Find the current line (last line whose time has passed)
        let index = -1;
        for (let i = 0; i < parsedLines.length; i++) {
            if (parsedLines[i].time <= adjustedTime) {
                index = i;
            } else {
                break;
            }
        }

        setCurrentLineIndex(index);
    }, [currentTime, parsedLines]);

    // Auto-scroll to current line
    const scrollToCurrentFn = () => {
        if (activeLineRef.current && containerRef.current) {
            const container = containerRef.current;
            const activeLine = activeLineRef.current;

            const containerHeight = container.clientHeight;
            const lineTop = activeLine.offsetTop;
            const lineHeight = activeLine.clientHeight;

            // Scroll so the active line is centered
            const scrollTo = lineTop - containerHeight / 2 + lineHeight / 2;

            container.scrollTo({
                top: scrollTo,
                behavior: 'smooth'
            });
        }
    };

    useEffect(() => {
        scrollToCurrentFn();
    }, [currentLineIndex]);

    // Handle resize (e.g. window resize or panel opening)
    useEffect(() => {
        if (!containerRef.current) return;

        const resizeObserver = new ResizeObserver(() => {
            scrollToCurrentFn();
        });

        resizeObserver.observe(containerRef.current);
        return () => resizeObserver.disconnect();
    }, [currentLineIndex]); // Re-bind if index changes (optional, but safe)

    if (parsedLines.length === 0) {
        return (
            <div className={cn("whitespace-pre-wrap text-2xl font-bold text-white/80 leading-relaxed space-y-8 pb-20 px-4", className)}>
                {lyrics}
            </div>
        );
    }

    return (
        <div ref={containerRef} className={cn("overflow-y-auto no-scrollbar pb-20", className)}>
            <div className="space-y-6 pt-[40vh] px-4">
                {parsedLines.map((line, index) => (
                    <div
                        key={index}
                        ref={index === currentLineIndex ? activeLineRef : null}
                        className={cn(
                            "transition-all duration-300 leading-relaxed cursor-pointer hover:text-white",
                            index === currentLineIndex
                                ? "text-3xl font-bold text-white scale-105"
                                : index < currentLineIndex
                                    ? "text-xl text-white/30"
                                    : "text-xl text-white/50"
                        )}
                    >
                        {line.text || '♪'}
                    </div>
                ))}
            </div>
        </div>
    );
}
