// Loading skeleton components
export function SongSkeleton() {
    return (
        <div className="flex items-center px-6 py-1.5 border-b border-gray-100 dark:border-white/5 h-14 animate-pulse">
            <div className="w-10 h-4 bg-gray-200 dark:bg-white/10 rounded" />
            <div className="w-12 h-9 bg-gray-200 dark:bg-white/10 rounded ml-4" />
            <div className="flex-1 ml-4 space-y-2">
                <div className="h-3 bg-gray-200 dark:bg-white/10 rounded w-1/3" />
                <div className="h-2 bg-gray-200 dark:bg-white/10 rounded w-1/4" />
            </div>
            <div className="w-16 h-3 bg-gray-200 dark:bg-white/10 rounded" />
        </div>
    );
}

export function AlbumSkeleton() {
    return (
        <div className="animate-pulse">
            <div className="aspect-square rounded-lg bg-gray-200 dark:bg-white/10 mb-3" />
            <div className="h-4 bg-gray-200 dark:bg-white/10 rounded w-3/4 mb-2" />
            <div className="h-3 bg-gray-200 dark:bg-white/10 rounded w-1/2" />
        </div>
    );
}

export function SongListSkeleton({ count = 10 }: { count?: number }) {
    return (
        <div>
            {Array.from({ length: count }).map((_, i) => (
                <SongSkeleton key={i} />
            ))}
        </div>
    );
}

export function AlbumGridSkeleton({ count = 12 }: { count?: number }) {
    return (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 p-8">
            {Array.from({ length: count }).map((_, i) => (
                <AlbumSkeleton key={i} />
            ))}
        </div>
    );
}
