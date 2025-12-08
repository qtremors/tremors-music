import { useEffect } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { useConfirmStore } from '../stores/confirmStore';
import { AlertTriangle, Trash2, Info } from 'lucide-react';

export function ConfirmDialog() {
    const { isOpen, options, handleConfirm, handleCancel } = useConfirmStore();

    // Handle keyboard events
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                handleCancel();
            } else if (e.key === 'Enter') {
                handleConfirm();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, handleConfirm, handleCancel]);

    const variant = options?.variant || 'default';

    const variantStyles = {
        danger: {
            icon: <Trash2 size={24} />,
            iconBg: 'bg-red-100 dark:bg-red-900/30',
            iconColor: 'text-red-600 dark:text-red-400',
            confirmBg: 'bg-red-600 hover:bg-red-700',
            confirmText: 'text-white',
        },
        warning: {
            icon: <AlertTriangle size={24} />,
            iconBg: 'bg-amber-100 dark:bg-amber-900/30',
            iconColor: 'text-amber-600 dark:text-amber-400',
            confirmBg: 'bg-amber-600 hover:bg-amber-700',
            confirmText: 'text-white',
        },
        default: {
            icon: <Info size={24} />,
            iconBg: 'bg-blue-100 dark:bg-blue-900/30',
            iconColor: 'text-blue-600 dark:text-blue-400',
            confirmBg: 'bg-apple-accent hover:bg-apple-accent/90',
            confirmText: 'text-white',
        },
    };

    const style = variantStyles[variant];

    return (
        <AnimatePresence>
            {isOpen && options && (
                <>
                    {/* Backdrop */}
                    <motion.div
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        onClick={handleCancel}
                        className="fixed inset-0 bg-black/50 backdrop-blur-sm z-[300]"
                    />

                    {/* Dialog */}
                    <motion.div
                        initial={{ opacity: 0, scale: 0.95, y: 10 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.95, y: 10 }}
                        transition={{ type: 'spring', duration: 0.3, bounce: 0.2 }}
                        className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-[301] w-full max-w-md"
                    >
                        <div className="bg-white dark:bg-[#2c2c2e] rounded-2xl shadow-2xl border border-gray-200 dark:border-white/10 overflow-hidden">
                            {/* Content */}
                            <div className="p-6">
                                <div className="flex items-start gap-4">
                                    {/* Icon */}
                                    <div className={`p-3 rounded-full ${style.iconBg} ${style.iconColor} flex-shrink-0`}>
                                        {style.icon}
                                    </div>

                                    {/* Text */}
                                    <div className="flex-1 min-w-0">
                                        <h3 className="text-lg font-semibold text-apple-text mb-1">
                                            {options.title}
                                        </h3>
                                        <p className="text-sm text-apple-subtext leading-relaxed">
                                            {options.message}
                                        </p>
                                    </div>
                                </div>
                            </div>

                            {/* Actions */}
                            <div className="flex gap-3 px-6 py-4 bg-gray-50 dark:bg-white/5 border-t border-gray-200 dark:border-white/10">
                                <button
                                    onClick={handleCancel}
                                    className="flex-1 px-4 py-2.5 rounded-xl bg-gray-200 dark:bg-white/10 text-apple-text font-medium hover:bg-gray-300 dark:hover:bg-white/15 transition-colors"
                                >
                                    {options.cancelText || 'Cancel'}
                                </button>
                                <button
                                    onClick={handleConfirm}
                                    className={`flex-1 px-4 py-2.5 rounded-xl font-medium transition-colors ${style.confirmBg} ${style.confirmText}`}
                                >
                                    {options.confirmText || 'Confirm'}
                                </button>
                            </div>
                        </div>
                    </motion.div>
                </>
            )}
        </AnimatePresence>
    );
}
