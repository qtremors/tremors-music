import { useState, useRef, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQueryClient } from '@tanstack/react-query';
import { MoreHorizontal, ListPlus, Disc, User, Play, Heart } from 'lucide-react';
import { usePlayerStore } from '../stores/playerStore';
import { useToastStore } from '../stores/toastStore';
import { toggleFavorite } from '../lib/api';

interface SongMenuProps {
    song: any;
    onAddToPlaylist: () => void;
    position?: 'left' | 'right';
}

export function SongContextMenu({ song, onAddToPlaylist }: SongMenuProps) {
    const [isOpen, setIsOpen] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { addToQueue } = usePlayerStore();
    const { addToast } = useToastStore();

    // Close menu when clicking outside
    useEffect(() => {
        const handleClickOutside = (e: MouseEvent) => {
            if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
                setIsOpen(false);
            }
        };
        if (isOpen) {
            document.addEventListener('mousedown', handleClickOutside);
            return () => document.removeEventListener('mousedown', handleClickOutside);
        }
    }, [isOpen]);

    const handleAddToQueue = () => {
        addToQueue(song);
        addToast('Added to queue');
        setIsOpen(false);
    };

    const handleGoToAlbum = () => {
        if (song.album_id) {
            navigate(`/albums/${song.album_id}`);
        }
        setIsOpen(false);
    };

    const handleGoToArtist = () => {
        if (song.artist) {
            navigate(`/artists/${encodeURIComponent(song.artist)}`);
        }
        setIsOpen(false);
    };

    const handleAddToPlaylist = () => {
        onAddToPlaylist();
        setIsOpen(false);
    };

    const handleToggleFavorite = async () => {
        await toggleFavorite(song.id);
        queryClient.invalidateQueries({ queryKey: ['songs'] });
        queryClient.invalidateQueries({ queryKey: ['smart-playlist'] });
        addToast(song.rating === 5 ? 'Removed from favorites' : 'Added to favorites');
        setIsOpen(false);
    };

    return (
        <div className="relative" ref={menuRef}>
            <button
                onClick={(e) => { e.stopPropagation(); setIsOpen(!isOpen); }}
                className="p-1.5 text-apple-subtext opacity-0 group-hover/row:opacity-100 transition rounded hover:bg-gray-300 dark:hover:bg-white/10"
            >
                <MoreHorizontal size={16} />
            </button>

            {isOpen && (
                <div
                    className="fixed w-48 bg-white dark:bg-[#2c2c2e] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 py-1.5 z-[100] animate-in fade-in zoom-in-95 duration-100"
                    style={{
                        top: menuRef.current ? menuRef.current.getBoundingClientRect().bottom + 4 : 0,
                        left: menuRef.current ? Math.min(menuRef.current.getBoundingClientRect().left, window.innerWidth - 200) : 0,
                    }}
                    onClick={(e) => e.stopPropagation()}
                >
                    <MenuItem icon={<Play size={14} />} label="Play Next" onClick={handleAddToQueue} />
                    <MenuItem
                        icon={<Heart size={14} fill={song.rating === 5 ? "currentColor" : "none"} />}
                        label={song.rating === 5 ? "Remove from Favorites" : "Add to Favorites"}
                        onClick={handleToggleFavorite}
                        highlight={song.rating === 5}
                    />
                    <MenuItem icon={<ListPlus size={14} />} label="Add to Playlist" onClick={handleAddToPlaylist} />

                    <div className="h-px bg-gray-100 dark:bg-white/10 my-1.5" />

                    {song.album_id && (
                        <MenuItem icon={<Disc size={14} />} label="Go to Album" onClick={handleGoToAlbum} />
                    )}
                    {song.artist && (
                        <MenuItem icon={<User size={14} />} label="Go to Artist" onClick={handleGoToArtist} />
                    )}
                </div>
            )}
        </div>
    );
}

function MenuItem({ icon, label, onClick, highlight }: { icon: React.ReactNode; label: string; onClick: () => void; highlight?: boolean }) {
    return (
        <button
            onClick={onClick}
            className="w-full flex items-center gap-3 px-3 py-2 text-sm text-apple-text hover:bg-gray-100 dark:hover:bg-white/10 transition"
        >
            <span className={highlight ? "text-pink-500" : "text-apple-subtext"}>{icon}</span>
            {label}
        </button>
    );
}

// Clickable link components for artist/album names
interface ClickableLinkProps {
    children: React.ReactNode;
    onClick: (e: React.MouseEvent) => void;
    className?: string;
}

export function ClickableLink({ children, onClick, className = '' }: ClickableLinkProps) {
    return (
        <span
            onClick={(e) => {
                e.stopPropagation();
                onClick(e);
            }}
            className={`hover:text-apple-accent hover:underline cursor-pointer transition ${className}`}
        >
            {children}
        </span>
    );
}

export function ArtistLink({ name, className = '' }: { name: string; className?: string }) {
    const navigate = useNavigate();

    if (!name) return null;

    return (
        <ClickableLink
            onClick={() => navigate(`/artists/${encodeURIComponent(name)}`)}
            className={className}
        >
            {name}
        </ClickableLink>
    );
}

export function AlbumLink({ id, title, className = '' }: { id: number; title: string; className?: string }) {
    const navigate = useNavigate();

    if (!id || !title) return null;

    return (
        <ClickableLink
            onClick={() => navigate(`/albums/${id}`)}
            className={className}
        >
            {title}
        </ClickableLink>
    );
}
