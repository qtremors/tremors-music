import { useState, useRef, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { getGenres } from '../lib/api';
import { cn } from '../lib/utils';
import { Music, ArrowUpDown, ArrowUp, ArrowDown } from 'lucide-react';
import { useFilterStore } from '../stores/filterStore';

export function GenresPage() {
    const navigate = useNavigate();
    const { genresSortBy: sortBy, genresSortOrder: order, setGenresSort } = useFilterStore();
    const [showMenu, setShowMenu] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);

    const { data: genres, isLoading } = useQuery({
        queryKey: ['genres'],
        queryFn: getGenres,
    });

    useEffect(() => {
        if (showMenu) {
            const handleClickOutside = (e: MouseEvent) => {
                if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                    setShowMenu(false);
                }
            };
            document.addEventListener('mousedown', handleClickOutside);
            return () => document.removeEventListener('mousedown', handleClickOutside);
        }
    }, [showMenu]);

    const sortedGenres = genres ? [...genres].sort((a, b) => {
        if (sortBy === 'name') {
            return order === 'asc'
                ? a.name.localeCompare(b.name)
                : b.name.localeCompare(a.name);
        }
        return order === 'asc'
            ? a.song_count - b.song_count
            : b.song_count - a.song_count;
    }) : [];

    const handleSort = (field: 'name' | 'song_count') => {
        if (sortBy === field) {
            setGenresSort(sortBy, order === 'asc' ? 'desc' : 'asc');
        } else {
            setGenresSort(field, field === 'name' ? 'asc' : 'desc');
        }
    };

    // Color palette for genre cards
    const colors = [
        'from-red-500 to-orange-500',
        'from-orange-500 to-yellow-500',
        'from-yellow-500 to-green-500',
        'from-green-500 to-teal-500',
        'from-teal-500 to-cyan-500',
        'from-cyan-500 to-blue-500',
        'from-blue-500 to-indigo-500',
        'from-indigo-500 to-purple-500',
        'from-purple-500 to-pink-500',
        'from-pink-500 to-red-500',
    ];

    if (isLoading) {
        return (
            <div className="h-full flex flex-col bg-apple-gray overflow-hidden">
                <header className="h-16 flex items-center px-8 border-b border-gray-200 dark:border-white/10 flex-shrink-0">
                    <h2 className="text-xl font-semibold text-apple-text">Genres</h2>
                </header>
                <div className="flex-1 p-8">
                    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
                        {Array.from({ length: 12 }).map((_, i) => (
                            <div key={i} className="aspect-square rounded-xl bg-gray-200 dark:bg-white/5 animate-pulse" />
                        ))}
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="h-full flex flex-col bg-apple-gray overflow-hidden">
            <header className="h-16 flex items-center justify-between px-8 border-b border-gray-200 dark:border-white/10 flex-shrink-0">
                <h2 className="text-xl font-semibold text-apple-text">Genres</h2>

                {/* Sort Menu */}
                <div className="relative" ref={menuRef}>
                    <button
                        onClick={() => setShowMenu(!showMenu)}
                        className={cn(
                            "flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium transition",
                            showMenu ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-gray-100 dark:hover:bg-white/5"
                        )}
                    >
                        <ArrowUpDown size={16} /> Sort
                    </button>
                    {showMenu && (
                        <div className="absolute right-0 top-10 w-48 bg-white dark:bg-[#2c2c2e] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 py-2 z-50 animate-in fade-in zoom-in-95 duration-100">
                            <button
                                onClick={() => handleSort('name')}
                                className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-white/5 flex items-center justify-between text-apple-text transition"
                            >
                                Name
                                {sortBy === 'name' && (order === 'asc' ? <ArrowUp size={14} /> : <ArrowDown size={14} />)}
                            </button>
                            <button
                                onClick={() => handleSort('song_count')}
                                className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 dark:hover:bg-white/5 flex items-center justify-between text-apple-text transition"
                            >
                                Song Count
                                {sortBy === 'song_count' && (order === 'asc' ? <ArrowUp size={14} /> : <ArrowDown size={14} />)}
                            </button>
                        </div>
                    )}
                </div>
            </header>

            <main className="flex-1 overflow-y-auto p-8 pb-32">
                {sortedGenres.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-apple-subtext">
                        <Music size={64} className="mb-4 opacity-30" />
                        <p>No genres found</p>
                        <p className="text-sm mt-1">Add music with genre tags to see them here</p>
                    </div>
                ) : (
                    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
                        {sortedGenres.map((genre, idx) => (
                            <div
                                key={genre.name}
                                onClick={() => navigate(`/genres/${encodeURIComponent(genre.name)}`)}
                                className="group cursor-pointer"
                            >
                                <div className={cn(
                                    "aspect-square rounded-xl shadow-lg flex flex-col items-center justify-center p-4 text-white bg-gradient-to-br transition-all duration-300 group-hover:scale-105 group-hover:shadow-xl",
                                    colors[idx % colors.length]
                                )}>
                                    <Music size={32} className="mb-2 opacity-80" />
                                    <h3 className="font-bold text-center truncate w-full">{genre.name}</h3>
                                    <p className="text-sm opacity-80 mt-1">{genre.song_count} songs</p>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </main>
        </div>
    );
}
