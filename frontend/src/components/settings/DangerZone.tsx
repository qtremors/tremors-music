// Danger zone component for destructive actions
import { AlertTriangle } from 'lucide-react';
import { resetLibrary } from '../../lib/api';
import { Card } from '../common/Card';
import { useConfirm } from '../../stores/confirmStore';
import { useToastStore } from '../../stores/toastStore';

export function DangerZone() {
    const confirm = useConfirm();
    const { addToast } = useToastStore();

    const handleReset = async () => {
        const confirmed = await confirm({
            title: 'Reset Library Database',
            message: 'This will wipe all songs and albums from the database. Your folder paths will be kept, and your actual music files will NOT be deleted. You will need to rescan your library after this.',
            confirmText: 'Reset Library',
            variant: 'danger',
        });

        if (!confirmed) return;

        try {
            await resetLibrary();
            addToast('Library reset successfully. Please rescan.', 'success');
            window.location.reload();
        } catch (e) {
            console.error('Failed to reset library:', e);
            addToast('Failed to reset library. Please try again.', 'error');
        }
    };

    return (
        <Card className="border-red-200 dark:border-red-900/50">
            <div className="space-y-3">
                <div className="flex items-center gap-2">
                    <AlertTriangle size={18} className="text-red-500" />
                    <h4 className="text-sm font-bold text-red-500 uppercase">Danger Zone</h4>
                </div>

                <p className="text-sm text-apple-subtext">
                    This will remove all songs and albums from the database but <strong>keep your folder paths</strong>.
                    Your music files will not be deleted.
                </p>

                <button
                    onClick={handleReset}
                    className="text-sm text-red-500 hover:text-red-600 hover:underline font-medium"
                >
                    Reset & Wipe Library Database
                </button>
            </div>
        </Card>
    );
}
