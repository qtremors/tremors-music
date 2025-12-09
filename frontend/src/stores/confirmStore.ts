import { create } from 'zustand';

interface ConfirmOptions {
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    variant?: 'danger' | 'warning' | 'default';
}

interface ConfirmState {
    isOpen: boolean;
    options: ConfirmOptions | null;
    resolve: ((value: boolean) => void) | null;
    showConfirm: (options: ConfirmOptions) => Promise<boolean>;
    handleConfirm: () => void;
    handleCancel: () => void;
}

export const useConfirmStore = create<ConfirmState>((set, get) => ({
    isOpen: false,
    options: null,
    resolve: null,

    showConfirm: (options: ConfirmOptions) => {
        return new Promise<boolean>((resolve) => {
            set({
                isOpen: true,
                options,
                resolve,
            });
        });
    },

    handleConfirm: () => {
        const { resolve } = get();
        if (resolve) resolve(true);
        set({ isOpen: false, options: null, resolve: null });
    },

    handleCancel: () => {
        const { resolve } = get();
        if (resolve) resolve(false);
        set({ isOpen: false, options: null, resolve: null });
    },
}));

// Helper hook for easy usage
export function useConfirm() {
    const showConfirm = useConfirmStore((state) => state.showConfirm);
    return showConfirm;
}
