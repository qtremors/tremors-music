import { useQuery } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { getArtists, getCoverUrl } from '../lib/api';
import { Mic2 } from 'lucide-react';

export function ArtistsPage() {
  const navigate = useNavigate();
  const { data: artists, isLoading } = useQuery({
    queryKey: ['artists'],
    queryFn: getArtists,
  });

  if (isLoading) return <div className="p-12 text-center text-apple-subtext">Loading Artists...</div>;

  return (
    <div className="h-full flex flex-col">
      <header className="h-16 flex items-center px-8 border-b border-gray-200 dark:border-white/10 bg-apple-gray/95 backdrop-blur z-10">
        <h2 className="text-xl font-semibold text-apple-text">Artists</h2>
      </header>
      
      <main className="flex-1 overflow-y-auto p-8 pb-32">
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-6">
          {artists?.map((artist) => (
            <div 
                key={artist.name} 
                className="group cursor-pointer text-center space-y-3"
                // Encode the artist name properly for the URL
                onClick={() => navigate(`/artists/${encodeURIComponent(artist.name)}`)}
            >
              <div className="aspect-square rounded-full overflow-hidden bg-gray-200 dark:bg-white/5 shadow-md border border-white/10 mx-auto w-full max-w-[200px] relative group-hover:shadow-lg transition-all">
                 {/* Artist Image */}
                 <img 
                   src={getCoverUrl(artist.cover_example)} 
                   className="w-full h-full object-cover group-hover:scale-110 transition duration-500"
                   onError={(e) => e.currentTarget.style.display = 'none'}
                 />
                 <div className="absolute inset-0 flex items-center justify-center -z-10">
                    <Mic2 size={40} className="text-gray-400" />
                 </div>
              </div>
              <div>
                <h3 className="font-medium text-apple-text truncate group-hover:text-apple-accent transition">{artist.name}</h3>
                <p className="text-xs text-apple-subtext">{artist.album_count} Albums</p>
              </div>
            </div>
          ))}
        </div>
      </main>
    </div>
  );
}