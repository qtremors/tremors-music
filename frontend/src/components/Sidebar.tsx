import { useState, useEffect } from 'react';
import { NavLink, useNavigate, useLocation } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { getPlaylists } from '../lib/api';
import { cn } from '../lib/utils';
import {
    Library, Settings, Disc, Mic2, Music, Tag,
    PlusCircle, ListMusic, Search, Heart, Sparkles, TrendingUp
} from 'lucide-react';

export function Sidebar({ setIsCreatingPlaylist }: { setIsCreatingPlaylist: (val: boolean) => void }) {
    const navigate = useNavigate();
    const location = useLocation();
    const [search, setSearch] = useState('');

    const { data: playlists } = useQuery({ queryKey: ['playlists'], queryFn: getPlaylists });

    useEffect(() => {
        const timer = setTimeout(() => {
            if (search.trim()) {
                navigate(`/search?q=${encodeURIComponent(search)}`);
            } else if (location.pathname === '/search' && !search.trim()) {
                navigate('/');
            }
        }, 300);
        return () => clearTimeout(timer);
    }, [search, navigate, location.pathname]);

    // eslint-disable-next-line react-hooks/set-state-in-effect -- Intentional: clear search when navigating away from search page
    useEffect(() => {
        if (location.pathname !== '/search') {
            setSearch('');
        }
    }, [location.pathname]);

    return (
        <div className="fixed top-0 left-0 w-64 h-full pb-28 bg-gray-50/80 dark:bg-black/50 border-r border-gray-200 dark:border-white/10 hidden lg:flex flex-col p-6 z-20 backdrop-blur-xl">

            <div className="mb-6 px-2 flex-shrink-0">
                <h1 className="text-2xl font-bold text-apple-accent flex items-center gap-2 mb-6 select-none">
                    <Library /> Tremors
                </h1>

                <div className="relative group">
                    <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-apple-subtext group-focus-within:text-apple-text transition-colors" />
                    <input
                        type="text"
                        placeholder="Search Library"
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="w-full bg-gray-200 dark:bg-white/10 rounded-lg pl-9 pr-3 py-2 text-sm text-apple-text outline-none focus:ring-2 focus:ring-apple-accent/50 transition-all placeholder:text-apple-subtext"
                    />
                </div>
            </div>

            <nav className="space-y-1 flex-shrink-0">
                <NavLink to="/" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
                    <Music size={18} /> Songs
                </NavLink>
                <NavLink to="/albums" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
                    <Disc size={18} /> Albums
                </NavLink>
                <NavLink to="/artists" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
                    <Mic2 size={18} /> Artists
                </NavLink>
                <NavLink to="/genres" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
                    <Tag size={18} /> Genres
                </NavLink>
            </nav>

            <div className="mt-6 mb-2 px-3 flex items-center justify-between group flex-shrink-0">
                <NavLink to="/playlists" className={({ isActive }) => cn("text-xs font-bold uppercase tracking-wider flex-1 transition", isActive ? "text-apple-accent" : "text-apple-subtext hover:text-apple-text")}>
                    Playlists
                </NavLink>
                <button
                    onClick={() => setIsCreatingPlaylist(true)}
                    className="hover:text-apple-accent text-apple-subtext transition p-1 rounded hover:bg-black/5 dark:hover:bg-white/10"
                    title="Create Playlist"
                >
                    <PlusCircle size={16} />
                </button>
            </div>

            <div className="space-y-1 overflow-y-auto flex-1 min-h-0 no-scrollbar -mx-2 px-2">
                {/* Smart Playlists */}
                <NavLink to="/smart/favorites" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition text-sm", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
                    <Heart size={16} className="text-pink-500 flex-shrink-0" /> Favorites
                </NavLink>
                <NavLink to="/smart/recently-added" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition text-sm", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
                    <Sparkles size={16} className="text-green-500 flex-shrink-0" /> Recently Added
                </NavLink>
                <NavLink to="/smart/most-played" className={({ isActive }) => cn("w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition text-sm", isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5")}>
                    <TrendingUp size={16} className="text-purple-500 flex-shrink-0" /> Most Played
                </NavLink>

                {/* Divider if there are user playlists */}
                {playlists && playlists.length > 0 && (
                    <div className="my-2 border-t border-gray-200 dark:border-white/10" />
                )}

                {/* User Playlists */}
                {playlists?.map(pl => (
                    <NavLink
                        key={pl.id}
                        to={`/playlists/${pl.id}`}
                        className={({ isActive }) => cn(
                            "w-full text-left px-3 py-2 rounded-md font-medium flex items-center gap-3 transition truncate text-sm",
                            isActive ? "bg-gray-200 dark:bg-white/10 text-apple-text" : "text-apple-subtext hover:text-apple-text hover:bg-white/5"
                        )}
                    >
                        <ListMusic size={16} className="flex-shrink-0" />
                        <span className="truncate">{pl.name}</span>
                    </NavLink>
                ))}
                {(!playlists || playlists.length === 0) && (
                    <div className="px-3 py-2 text-xs text-apple-subtext text-center italic">
                        No custom playlists yet
                    </div>
                )}
            </div>

            <div className="mt-4 pt-4 border-t border-gray-200 dark:border-white/5 flex-shrink-0">
                <NavLink to="/settings" className={({ isActive }) => cn("flex items-center gap-2 text-sm font-medium transition p-3 rounded-lg w-full", isActive ? "bg-apple-accent/10 text-apple-accent" : "text-apple-subtext hover:text-apple-text hover:bg-black/5 dark:hover:bg-white/5")}>
                    <Settings size={18} /> Settings
                </NavLink>
            </div>
        </div>
    );
}
