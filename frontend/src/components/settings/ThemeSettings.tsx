// Theme settings component
import { Moon, Sun, Check } from 'lucide-react';
import { useThemeStore } from '../../stores/themeStore';
import { Card } from '../common/Card';

export function ThemeSettings() {
    const { theme, setTheme, accentColor, setAccentColor, showSongListArt, setShowSongListArt } = useThemeStore();

    // Colors from current version
    const colors = [
        { name: 'Red', val: '250 45 72' },
        { name: 'Blue', val: '0 122 255' },
        { name: 'Purple', val: '175 82 222' },
        { name: 'Orange', val: '255 149 0' },
        { name: 'Teal', val: '48 176 199' },
        { name: 'Green', val: '40 205 65' },
    ];

    return (
        <Card>
            <div className="space-y-6">
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

                {/* Show Album Art Toggle */}
                <div className="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-white/5">
                    <span className="text-apple-text font-medium">Show Art in List</span>
                    <button
                        onClick={() => setShowSongListArt(!showSongListArt)}
                        className={`w-12 h-6 rounded-full transition-colors relative ${showSongListArt ? 'bg-apple-accent' : 'bg-gray-300 dark:bg-gray-600'}`}
                    >
                        <div className={`absolute top-1 w-4 h-4 bg-white rounded-full shadow-sm transition-transform ${showSongListArt ? 'left-7' : 'left-1'}`} />
                    </button>
                </div>
            </div>
        </Card>
    );
}
