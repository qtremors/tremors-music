// Library path manager component with CRUD operations
import { useState, useEffect, useCallback } from 'react';
import { Plus, Trash2, Edit2, Check, X } from 'lucide-react';
import api from '../../lib/api';
import { Card } from '../common/Card';
import { IconButton } from '../common/IconButton';
import { useConfirm } from '../../stores/confirmStore';

interface LibraryPath {
    id: number;
    path: string;
}

export function LibraryPathManager() {
    const [paths, setPaths] = useState<LibraryPath[]>([]);
    const [newPath, setNewPath] = useState('');
    const [editingId, setEditingId] = useState<number | null>(null);
    const [editingPath, setEditingPath] = useState('');
    const confirm = useConfirm();

    const loadPaths = useCallback(async () => {
        try {
            const res = await api.get<LibraryPath[]>('/library/paths');
            setPaths(Array.isArray(res.data) ? res.data : []);
        } catch {
            console.error('Failed to load paths');
        }
    }, []);

    useEffect(() => {
        loadPaths();
    }, [loadPaths]);

    const handleAddPath = async () => {
        if (!newPath.trim()) return;
        try {
            await api.post('/library/paths', { path: newPath });
            setNewPath('');
            loadPaths();
        } catch {
            alert('Invalid path or server error');
        }
    };

    const handleRemovePath = async (id: number) => {
        const confirmed = await confirm({
            title: 'Remove Library Path',
            message: 'Remove this folder from your library? Your files will not be deleted.',
            confirmText: 'Remove',
            variant: 'warning',
        });

        if (!confirmed) return;
        try {
            await api.delete(`/library/paths/${id}`);
            loadPaths();
        } catch {
            console.error('Failed to remove path');
        }
    };

    const startEditing = (path: LibraryPath) => {
        setEditingId(path.id);
        setEditingPath(path.path);
    };

    const cancelEditing = () => {
        setEditingId(null);
        setEditingPath('');
    };

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const saveEdit = async (_id: number) => {
        if (!editingPath.trim()) return;
        // Note: Backend doesn't have edit endpoint yet, would need to add
        // For now, just cancel editing
        cancelEditing();
        // TODO: Implement backend PATCH /library/paths/{id}
    };


    return (
        <Card>
            <div className="space-y-4">
                <h4 className="text-sm font-semibold text-apple-subtext uppercase tracking-wider">Music Folders</h4>

                {/* Path List */}
                <div className="space-y-2">
                    {paths.length === 0 && (
                        <p className="text-sm text-apple-subtext italic">No folders added yet.</p>
                    )}
                    {paths.map(p => (
                        <div
                            key={p.id}
                            className="flex items-center justify-between bg-gray-100 dark:bg-white/5 p-3 rounded-lg border border-transparent hover:border-gray-200 dark:hover:border-white/10 transition"
                        >
                            {editingId === p.id ? (
                                <>
                                    <input
                                        type="text"
                                        value={editingPath}
                                        onChange={(e) => setEditingPath(e.target.value)}
                                        className="flex-1 px-2 py-1 text-sm bg-white dark:bg-black/20 border border-gray-300 dark:border-white/20 rounded text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent/50"
                                        autoFocus
                                    />
                                    <div className="flex gap-1 ml-2">
                                        <IconButton
                                            icon={<Check size={16} />}
                                            onClick={() => saveEdit(p.id)}
                                            variant="accent"
                                            size="sm"
                                            tooltip="Save"
                                        />
                                        <IconButton
                                            icon={<X size={16} />}
                                            onClick={cancelEditing}
                                            variant="ghost"
                                            size="sm"
                                            tooltip="Cancel"
                                        />
                                    </div>
                                </>
                            ) : (
                                <>
                                    <span className="text-sm font-mono text-apple-text truncate px-2 flex-1">{p.path}</span>
                                    <div className="flex gap-1">
                                        <IconButton
                                            icon={<Edit2 size={16} />}
                                            onClick={() => startEditing(p)}
                                            variant="ghost"
                                            size="sm"
                                            tooltip="Edit path"
                                        />
                                        <IconButton
                                            icon={<Trash2 size={16} />}
                                            onClick={() => handleRemovePath(p.id)}
                                            variant="ghost"
                                            size="sm"
                                            tooltip="Remove path"
                                            className="hover:text-red-500"
                                        />
                                    </div>
                                </>
                            )}
                        </div>
                    ))}
                </div>

                {/* Add New Path */}
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={newPath}
                        onChange={(e) => setNewPath(e.target.value)}
                        onKeyDown={(e) => e.key === 'Enter' && handleAddPath()}
                        placeholder="e.g. D:/Music or /home/user/Music"
                        className="flex-1 p-3 text-sm rounded-lg border border-gray-200 dark:border-white/10 bg-white dark:bg-black/20 text-apple-text focus:outline-none focus:ring-2 focus:ring-apple-accent/50"
                    />
                    <IconButton
                        icon={<Plus size={20} />}
                        onClick={handleAddPath}
                        variant="default"
                        tooltip="Add folder"
                    />
                </div>
            </div>
        </Card>
    );
}
