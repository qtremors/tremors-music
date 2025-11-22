import { useState, useEffect } from 'react';
import { Plus, Trash2, RefreshCw, Moon, Sun, Check } from 'lucide-react';
import api, { resetLibrary } from '../lib/api';
import { useThemeStore } from '../stores/themeStore';

interface LibraryPath {
  id: number;
  path: string;
}

export function SettingsPage() {
  const [paths, setPaths] = useState<LibraryPath[]>([]);
  const [newPath, setNewPath] = useState('');
  const [isScanning, setIsScanning] = useState(false);
  
  const { theme, setTheme, accentColor, setAccentColor, showSongListArt, setShowSongListArt } = useThemeStore();

  useEffect(() => {
    loadPaths();
  }, []);

  const loadPaths = async () => {
    try {
      const res = await api.get<LibraryPath[]>('/library/paths');
      setPaths(res.data);
    } catch (e) { console.error(e); }
  };

  const handleAddPath = async () => {
    if (!newPath) return;
    try {
      await api.post('/library/paths', { path: newPath });
      setNewPath('');
      loadPaths();
    } catch (e) {
      alert('Invalid path or server error');
    }
  };

  const handleRemovePath = async (id: number) => {
    await api.delete(`/library/paths/${id}`);
    loadPaths();
  };

  const handleRescan = async () => {
    setIsScanning(true);
    await api.post('/library/scan');
    setTimeout(() => setIsScanning(false), 2000);
  };

  const handleReset = async () => {
    if (confirm("Are you sure? This will wipe all songs/albums. You will need to rescan.")) {
        await resetLibrary();
        window.location.reload();
    }
  };

  // Colors for the picker (RGB values)
  const colors = [
    { name: 'Red', val: '250 45 72' },
    { name: 'Blue', val: '0 122 255' },
    { name: 'Purple', val: '175 82 222' },
    { name: 'Orange', val: '255 149 0' },
    { name: 'Teal', val: '48 176 199' },
    { name: 'Green', val: '40 205 65' },
  ];

  return (
    <div className="h-full flex flex-col overflow-y-auto bg-apple-gray/50">
      <header className="h-16 glass flex items-center px-8 sticky top-0 z-10 border-b border-white/10">
        <h2 className="text-xl font-semibold text-apple-text">Settings</h2>
      </header>
      
      <div className="p-8 max-w-3xl mx-auto w-full space-y-8 pb-32">
        
        {/* Appearance Section */}
        <section>
          <h3 className="text-sm font-semibold text-apple-subtext uppercase mb-4 tracking-wider">Appearance</h3>
          <div className="glass-panel p-6 space-y-6">
            
            {/* Theme Toggle */}
            <div className="flex items-center justify-between">
              <span className="text-apple-text font-medium">App Theme</span>
              <div className="bg-gray-200 dark:bg-gray-700 rounded-full p-1 flex gap-1">
                <button 
                  onClick={() => setTheme('light')}
                  className={`p-2 rounded-full transition-all ${theme === 'light' ? 'bg-white text-black shadow-sm' : 'text-gray-500'}`}
                >
                  <Sun size={18} />
                </button>
                <button 
                  onClick={() => setTheme('dark')}
                  className={`p-2 rounded-full transition-all ${theme === 'dark' ? 'bg-gray-600 text-white shadow-sm' : 'text-gray-500'}`}
                >
                  <Moon size={18} />
                </button>
              </div>
            </div>

            <hr className="border-gray-200 dark:border-white/5" />

            {/* Accent Color */}
            <div className="space-y-3">
              <span className="text-apple-text font-medium">Accent Color</span>
              <div className="flex flex-wrap gap-4">
                {colors.map((c) => (
                  <button
                    key={c.name}
                    onClick={() => setAccentColor(c.val)}
                    className={`w-10 h-10 rounded-full flex items-center justify-center transition-transform hover:scale-105 ${accentColor === c.val ? 'ring-2 ring-offset-2 ring-gray-400 dark:ring-offset-gray-800' : ''}`}
                    style={{ backgroundColor: `rgb(${c.val})` }}
                  >
                    {accentColor === c.val && <Check size={16} className="text-white drop-shadow-md" />}
                  </button>
                ))}
              </div>
            </div>

            <div className="flex items-center justify-between mt-4 pt-4 border-t border-gray-200 dark:border-white/5">
                <span className="text-apple-text font-medium">Show Art in List</span>
                <button 
                    onClick={() => setShowSongListArt(!showSongListArt)}
                    className={`w-12 h-6 rounded-full transition-colors relative ${showSongListArt ? 'bg-apple-accent' : 'bg-gray-300 dark:bg-gray-600'}`}
                >
                    <div className={`absolute top-1 w-4 h-4 bg-white rounded-full shadow-sm transition-transform ${showSongListArt ? 'left-7' : 'left-1'}`} />
                </button>
            </div>
          </div>
        </section>

        {/* Library Management Section */}
        <section>
          <h3 className="text-sm font-semibold text-apple-subtext uppercase mb-4 tracking-wider">Library Management</h3>
          <div className="glass-panel p-6">
            <div className="space-y-2 mb-6">
              {paths.length === 0 && <p className="text-sm text-apple-subtext italic">No folders added yet.</p>}
              {paths.map(p => (
                <div key={p.id} className="flex items-center justify-between bg-gray-100 dark:bg-white/5 p-3 rounded-lg border border-transparent hover:border-gray-200 dark:hover:border-white/10 transition">
                  <span className="text-sm font-mono text-apple-text truncate px-2">{p.path}</span>
                  <button onClick={() => handleRemovePath(p.id)} className="text-apple-subtext hover:text-red-500 p-2 transition">
                    <Trash2 size={18} />
                  </button>
                </div>
              ))}
            </div>

            <div className="flex gap-2 mb-8">
              <input 
                type="text" 
                value={newPath}
                onChange={(e) => setNewPath(e.target.value)}
                placeholder="e.g. D:/Music"
                className="flex-1 p-3 text-sm rounded-lg border border-gray-200 dark:border-white/10 bg-white dark:bg-black/20 text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent/50"
              />
              <button onClick={handleAddPath} className="p-3 bg-gray-100 dark:bg-white/10 rounded-lg hover:bg-gray-200 dark:hover:bg-white/20 transition text-apple-text">
                <Plus size={20} />
              </button>
            </div>

            <button 
              onClick={handleRescan}
              disabled={isScanning}
              className="w-full py-3 bg-apple-accent text-white rounded-lg font-medium flex items-center justify-center gap-2 hover:opacity-90 transition shadow-lg shadow-apple-accent/20"
            >
              <RefreshCw size={18} className={isScanning ? "animate-spin" : ""} />
              {isScanning ? 'Scanning Library...' : 'Rescan All Folders'}
            </button>

            <div className="mt-4 pt-4 border-t border-gray-200 dark:border-white/5">
                <h4 className="text-xs font-bold text-red-500 uppercase mb-2">Danger Zone</h4>
                <button 
                    onClick={handleReset}
                    className="text-sm text-red-500 hover:text-red-600 hover:underline"
                >
                    Reset & Wipe Library Database
                </button>
                <p className="text-xs text-apple-subtext mt-1">This will remove all songs/albums but <b>keep your folder paths</b>.</p>
            </div>
          </div>
        </section>

      </div>
    </div>
  );
}