import { ThemeSettings } from '../components/settings/ThemeSettings';
import { LibraryPathManager } from '../components/settings/LibraryPathManager';
import { ScannerControl } from '../components/settings/ScannerControl';
import { DangerZone } from '../components/settings/DangerZone';

export function SettingsPage() {
  return (
    <div className="h-full flex flex-col overflow-y-auto bg-apple-gray/50">
      <header className="h-16 glass flex items-center px-8 sticky top-0 z-10 border-b border-white/10">
        <h2 className="text-xl font-semibold text-apple-text">Settings</h2>
      </header>

      <div className="p-8 max-w-3xl mx-auto w-full space-y-8 pb-32">

        {/* Appearance Section */}
        <section>
          <h3 className="text-sm font-semibold text-apple-subtext uppercase mb-4 tracking-wider">Appearance</h3>
          <ThemeSettings />
        </section>

        {/* Library Management Section */}
        <section>
          <h3 className="text-sm font-semibold text-apple-subtext uppercase mb-4 tracking-wider">Library Management</h3>
          <div className="space-y-4">
            <LibraryPathManager />
            <ScannerControl />
          </div>
        </section>

        {/* Danger Zone Section */}
        <section>
          <h3 className="text-sm font-semibold text-apple-subtext uppercase mb-4 tracking-wider">Danger Zone</h3>
          <DangerZone />
        </section>

      </div>
    </div>
  );
}