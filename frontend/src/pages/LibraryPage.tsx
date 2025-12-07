import { SongList } from '../components/SongList';
import { PageWrapper } from '../components/PageWrapper';

export function LibraryPage() {
  return (
    <PageWrapper>
      <div className="h-full flex flex-col">
        <header className="h-16 flex items-center px-8 border-b border-gray-200 dark:border-white/10 flex-shrink-0 bg-apple-gray/95 backdrop-blur z-10">
          <h2 className="text-xl font-semibold text-apple-text">Library</h2>
        </header>

        <main className="flex-1 relative min-h-0">
          <SongList />
        </main>
      </div>
    </PageWrapper>
  );
}