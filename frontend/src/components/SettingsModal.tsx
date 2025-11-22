import { useState, useEffect } from 'react';
import { X, Plus, Trash2, RefreshCw, Moon, Sun } from 'lucide-react';
import api from '../lib/api';
import { useThemeStore } from '../stores/themeStore';

interface SettingsModalProps {
  onClose: () => void;
}

interface LibraryPath {
  id: number;
  path: string;
}

export function SettingsModal({ onClose }: SettingsModalProps) {
  const [paths, setPaths] = useState<LibraryPath[]>([]);
  const [newPath, setNewPath] = useState('');
  const [isScanning, setIsScanning] = useState(false);
  
  const { theme, setTheme, accentColor, setAccentColor } = useThemeStore();

  useEffect(() => {
    loadPaths();
  }, []);

  const loadPaths = async () => {
    const res = await api.get<LibraryPath[]>('/library/paths');
    setPaths(res.data);
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

  // Colors for the picker (RGB values)
  const colors = [
    { name: 'Red', val: '250 45 72' },
    { name: 'Blue', val: '0 122 255' },
    { name: 'Purple', val: '175 82 222' },
    { name: 'Orange', val: '255 149 0' },
    { name: 'Teal', val: '48 176 199' },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/20 backdrop-blur-sm">
      <div className="glass-panel w-full max-w-md p-6 relative animate-in fade-in zoom-in duration-200">
        <button onClick={onClose} className="absolute top-4 right-4 text-apple-subtext hover:text-apple-text">
          <X size={20} />
        </button>

        <h2 className="text-xl font-bold mb-6 text-apple-text">Settings</h2>

        {/* --- Appearance Section --- */}
        <div className="mb-8">
          <h3 className="text-sm font-semibold text-apple-subtext uppercase mb-3">Appearance</h3>
          
          <div className="flex items-center justify-between mb-4">
            <span className="text-apple-text">Theme</span>
            <div className="bg-gray-200 dark:bg-gray-700 rounded-full p-1 flex gap-1">
              <button 
                onClick={() => setTheme('light')}
                className={`p-2 rounded-full transition ${theme === 'light' ? 'bg-white shadow-sm' : 'text-gray-500'}`}
              >
                <Sun size={16} />
              </button>
              <button 
                onClick={() => setTheme('dark')}
                className={`p-2 rounded-full transition ${theme === 'dark' ? 'bg-gray-600 text-white shadow-sm' : 'text-gray-500'}`}
              >
                <Moon size={16} />
              </button>
            </div>
          </div>

          <div className="flex gap-3">
            {colors.map((c) => (
              <button
                key={c.name}
                onClick={() => setAccentColor(c.val)}
                className={`w-8 h-8 rounded-full border-2 transition ${accentColor === c.val ? 'border-gray-400 scale-110' : 'border-transparent'}`}
                style={{ backgroundColor: `rgb(${c.val})` }}
              />
            ))}
          </div>
        </div>

        {/* --- Library Section --- */}
        <div>
          <h3 className="text-sm font-semibold text-apple-subtext uppercase mb-3">Library Paths</h3>
          
          <div className="space-y-2 mb-4 max-h-40 overflow-y-auto">
            {paths.map(p => (
              <div key={p.id} className="flex items-center justify-between bg-gray-100 dark:bg-gray-800 p-3 rounded-lg">
                <span className="text-xs font-mono truncate max-w-[200px] text-apple-text">{p.path}</span>
                <button onClick={() => handleRemovePath(p.id)} className="text-red-500 hover:bg-red-100 p-1 rounded">
                  <Trash2 size={16} />
                </button>
              </div>
            ))}
          </div>

          <div className="flex gap-2 mb-6">
            <input 
              type="text" 
              value={newPath}
              onChange={(e) => setNewPath(e.target.value)}
              placeholder="Paste folder path..."
              className="flex-1 p-2 text-sm rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent"
            />
            <button onClick={handleAddPath} className="p-2 bg-gray-200 dark:bg-gray-700 rounded-lg hover:bg-gray-300">
              <Plus size={20} className="text-apple-text" />
            </button>
          </div>

          <button 
            onClick={handleRescan}
            disabled={isScanning}
            className="w-full py-3 bg-apple-accent text-white rounded-lg font-medium flex items-center justify-center gap-2 hover:opacity-90 transition"
          >
            <RefreshCw size={18} className={isScanning ? "animate-spin" : ""} />
            {isScanning ? 'Scanning...' : 'Rescan Library'}
          </button>
        </div>

      </div>
    </div>
  );
}