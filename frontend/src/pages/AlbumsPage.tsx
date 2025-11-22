import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { getAlbums, getCoverUrl } from '../lib/api';
import { Disc } from 'lucide-react';

// --- Sub-Component to handle Image State ---
function AlbumCard({ album }: { album: any }) {
  const navigate = useNavigate();
  const [imageError, setImageError] = useState(false);

  return (
    <div 
      onClick={() => navigate(`/albums/${album.id}`)}
      className="group cursor-pointer space-y-3"
    >
      <div className="aspect-square bg-gray-200 dark:bg-white/5 rounded-lg shadow-sm overflow-hidden relative border border-black/5 dark:border-white/5 group-hover:shadow-md transition-all">
        {/* Condition: Show Image if NO error */}
        {!imageError ? (
          <img 
            src={getCoverUrl(album.id)} 
            alt={album.title}
            className="w-full h-full object-cover transition duration-500 group-hover:scale-105"
            loading="lazy"
            onError={() => setImageError(true)} // Switch state on error
          />
        ) : (
          // Condition: Show Fallback if error
          <div className="w-full h-full flex items-center justify-center text-gray-400 bg-gray-100 dark:bg-white/5">
            <Disc size={40} />
          </div>
        )}
      </div>
      
      <div>
        <h3 className="text-sm font-medium text-apple-text truncate group-hover:text-apple-accent transition-colors">
          {album.title}
        </h3>
        <p className="text-xs text-apple-subtext truncate">{album.artist}</p>
      </div>
    </div>
  );
}

// --- Main Page ---
export function AlbumsPage() {
  const { data: albums, isLoading } = useQuery({
    queryKey: ['albums'],
    queryFn: () => getAlbums(0, 1000),
  });

  if (isLoading) return <div className="p-12 text-center text-apple-subtext">Loading albums...</div>;

  return (
    <div className="h-full flex flex-col bg-apple-gray">
      <header className="h-16 flex items-center px-8 sticky top-0 z-10 border-b border-gray-200 dark:border-white/10 bg-apple-gray/95 backdrop-blur flex-shrink-0">
        <h2 className="text-xl font-semibold text-apple-text">Albums</h2>
      </header>

      <main className="flex-1 overflow-y-auto p-8 pb-32">
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6">
          {albums?.map((album) => (
            <AlbumCard key={album.id} album={album} />
          ))}
        </div>
      </main>
    </div>
  );
}