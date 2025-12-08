// Loading skeleton components with pulse animation

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

export function ArtistSkeleton() {
    return (
        <div className="animate-pulse text-center space-y-3">
            <div className="aspect-square rounded-full bg-gray-200 dark:bg-white/10 mx-auto w-full max-w-[200px]" />
            <div className="h-4 bg-gray-200 dark:bg-white/10 rounded w-3/4 mx-auto" />
            <div className="h-3 bg-gray-200 dark:bg-white/10 rounded w-1/2 mx-auto" />
        </div>
    );
}

export function PlaylistSkeleton() {
    return (
        <div className="animate-pulse">
            <div className="aspect-square rounded-lg bg-gray-200 dark:bg-white/10 mb-3" />
            <div className="h-4 bg-gray-200 dark:bg-white/10 rounded w-3/4 mb-2" />
            <div className="h-3 bg-gray-200 dark:bg-white/10 rounded w-1/3" />
        </div>
    );
}

export function SearchResultSkeleton() {
    return (
        <div className="animate-pulse flex items-center gap-3 px-4 py-3">
            <div className="w-12 h-12 rounded bg-gray-200 dark:bg-white/10 flex-shrink-0" />
            <div className="flex-1 space-y-2">
                <div className="h-3 bg-gray-200 dark:bg-white/10 rounded w-1/2" />
                <div className="h-2 bg-gray-200 dark:bg-white/10 rounded w-1/3" />
            </div>
            <div className="w-10 h-3 bg-gray-200 dark:bg-white/10 rounded" />
        </div>
    );
}

export function TopResultSkeleton() {
    return (
        <div className="animate-pulse flex items-center gap-6 p-6 rounded-xl bg-gray-100 dark:bg-white/5 border border-gray-200 dark:border-white/10">
            <div className="w-32 h-32 rounded-lg bg-gray-200 dark:bg-white/10 flex-shrink-0" />
            <div className="flex-1 space-y-4">
                <div className="h-8 bg-gray-200 dark:bg-white/10 rounded w-1/3" />
                <div className="h-4 bg-gray-200 dark:bg-white/10 rounded w-1/4" />
                <div className="h-6 bg-gray-200 dark:bg-white/10 rounded w-16" />
            </div>
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

export function ArtistGridSkeleton({ count = 10 }: { count?: number }) {
    return (
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-6 p-8">
            {Array.from({ length: count }).map((_, i) => (
                <ArtistSkeleton key={i} />
            ))}
        </div>
    );
}

export function PlaylistGridSkeleton({ count = 8 }: { count?: number }) {
    return (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6 p-8">
            {Array.from({ length: count }).map((_, i) => (
                <PlaylistSkeleton key={i} />
            ))}
        </div>
    );
}

export function SearchPageSkeleton() {
    return (
        <div className="space-y-8 p-8 animate-pulse">
            {/* Top Result Skeleton */}
            <div>
                <div className="h-5 bg-gray-200 dark:bg-white/10 rounded w-28 mb-4" />
                <TopResultSkeleton />
            </div>

            {/* Songs Section */}
            <div>
                <div className="h-5 bg-gray-200 dark:bg-white/10 rounded w-20 mb-4" />
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                    {Array.from({ length: 4 }).map((_, i) => (
                        <SearchResultSkeleton key={i} />
                    ))}
                </div>
            </div>

            {/* Albums Section */}
            <div>
                <div className="h-5 bg-gray-200 dark:bg-white/10 rounded w-20 mb-4" />
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
                    {Array.from({ length: 6 }).map((_, i) => (
                        <AlbumSkeleton key={i} />
                    ))}
                </div>
            </div>
        </div>
    );
}

export function DetailHeaderSkeleton() {
    return (
        <div className="animate-pulse flex items-end gap-6 p-8 pb-6 border-b border-gray-200 dark:border-white/10">
            <div className="w-32 h-32 sm:w-40 sm:h-40 bg-gray-200 dark:bg-white/10 rounded-lg" />
            <div className="flex-1 space-y-3 mb-2">
                <div className="h-3 bg-gray-200 dark:bg-white/10 rounded w-16" />
                <div className="h-8 bg-gray-200 dark:bg-white/10 rounded w-1/3" />
                <div className="h-4 bg-gray-200 dark:bg-white/10 rounded w-1/4" />
            </div>
        </div>
    );
}

export function ArtistDetailSkeleton() {
    return (
        <div className="h-full flex flex-col bg-apple-gray animate-pulse">
            {/* Header */}
            <div className="p-8 pb-4 space-y-4">
                <div className="h-4 bg-gray-200 dark:bg-white/10 rounded w-20" />
                <div className="flex items-end justify-between">
                    <div className="space-y-2">
                        <div className="h-10 bg-gray-200 dark:bg-white/10 rounded w-48" />
                        <div className="h-4 bg-gray-200 dark:bg-white/10 rounded w-32" />
                    </div>
                    <div className="flex gap-3">
                        <div className="w-10 h-10 bg-gray-200 dark:bg-white/10 rounded-full" />
                        <div className="w-10 h-10 bg-gray-200 dark:bg-white/10 rounded-full" />
                    </div>
                </div>
            </div>

            {/* Albums */}
            <div className="px-8 space-y-12">
                {Array.from({ length: 2 }).map((_, idx) => (
                    <div key={idx} className="flex flex-col md:flex-row gap-8">
                        <div className="md:w-48 flex-shrink-0">
                            <div className="w-40 h-40 bg-gray-200 dark:bg-white/10 rounded-lg" />
                            <div className="mt-3 space-y-2">
                                <div className="h-5 bg-gray-200 dark:bg-white/10 rounded w-3/4" />
                                <div className="h-3 bg-gray-200 dark:bg-white/10 rounded w-1/2" />
                            </div>
                        </div>
                        <div className="flex-1 space-y-2">
                            {Array.from({ length: 4 }).map((_, i) => (
                                <div key={i} className="flex items-center px-4 py-3">
                                    <div className="w-8 h-4 bg-gray-200 dark:bg-white/10 rounded" />
                                    <div className="flex-1 px-4">
                                        <div className="h-4 bg-gray-200 dark:bg-white/10 rounded w-1/2" />
                                    </div>
                                    <div className="w-12 h-4 bg-gray-200 dark:bg-white/10 rounded" />
                                </div>
                            ))}
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}

export function AlbumDetailSkeleton() {
    return (
        <div className="h-full flex flex-col bg-apple-gray animate-pulse">
            {/* Hero Section */}
            <div className="relative w-full h-80 overflow-hidden flex items-end p-8">
                <div className="absolute inset-0 bg-gray-200 dark:bg-white/10" />
                <div className="relative z-10 flex items-end gap-8 w-full">
                    <div className="w-48 h-48 bg-gray-300 dark:bg-white/20 rounded-lg" />
                    <div className="flex-1 mb-2 space-y-3">
                        <div className="h-4 bg-gray-300 dark:bg-white/20 rounded w-16" />
                        <div className="h-10 bg-gray-300 dark:bg-white/20 rounded w-1/3" />
                        <div className="h-5 bg-gray-300 dark:bg-white/20 rounded w-1/4" />
                    </div>
                    <div className="flex gap-3 mb-2">
                        <div className="w-14 h-14 bg-gray-300 dark:bg-white/20 rounded-full" />
                        <div className="w-14 h-14 bg-gray-300 dark:bg-white/20 rounded-full" />
                    </div>
                </div>
            </div>

            {/* Song List */}
            <div className="px-8 mt-4">
                <div className="flex items-center px-4 py-2 border-b border-gray-200 dark:border-white/10 mb-2">
                    <div className="w-10 h-3 bg-gray-200 dark:bg-white/10 rounded" />
                    <div className="flex-1 h-3 bg-gray-200 dark:bg-white/10 rounded w-12 ml-4" />
                    <div className="w-16 h-3 bg-gray-200 dark:bg-white/10 rounded" />
                </div>
                <div className="space-y-1">
                    {Array.from({ length: 8 }).map((_, i) => (
                        <div key={i} className="flex items-center px-4 py-3">
                            <div className="w-10 h-4 bg-gray-200 dark:bg-white/10 rounded" />
                            <div className="flex-1 h-4 bg-gray-200 dark:bg-white/10 rounded w-1/3 ml-4" />
                            <div className="w-12 h-4 bg-gray-200 dark:bg-white/10 rounded" />
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}

export function PlaylistDetailSkeleton() {
    return (
        <div className="h-full flex flex-col bg-apple-gray animate-pulse">
            <DetailHeaderSkeleton />
            <div className="flex-1 px-8 py-4">
                <div className="flex items-center px-4 py-2 mb-2">
                    <div className="w-8 h-3 bg-gray-200 dark:bg-white/10 rounded" />
                    <div className="w-12 h-3 bg-gray-200 dark:bg-white/10 rounded ml-4" />
                    <div className="flex-1 h-3 bg-gray-200 dark:bg-white/10 rounded w-12 ml-4" />
                    <div className="w-16 h-3 bg-gray-200 dark:bg-white/10 rounded" />
                </div>
                {Array.from({ length: 8 }).map((_, i) => (
                    <SongSkeleton key={i} />
                ))}
            </div>
        </div>
    );
}

