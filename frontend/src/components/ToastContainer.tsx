import { AnimatePresence, motion } from 'framer-motion';
import { useToastStore } from '../stores/toastStore';
import { CheckCircle, AlertCircle, Info } from 'lucide-react';

export function ToastContainer() {
  const { toasts } = useToastStore();

  return (
    <div className="fixed bottom-24 right-8 z-[200] flex flex-col gap-3 pointer-events-none">
      <AnimatePresence>
        {toasts.map((toast) => (
          <motion.div
            key={toast.id}
            initial={{ opacity: 0, y: 20, scale: 0.9 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 20, scale: 0.9 }}
            layout
            className="bg-white/90 dark:bg-[#2c2c2e]/90 backdrop-blur-md border border-gray-200 dark:border-white/10 text-apple-text px-4 py-3 rounded-xl shadow-2xl flex items-center gap-3 min-w-[200px] pointer-events-auto"
          >
            {toast.type === 'success' && <CheckCircle size={18} className="text-green-500" />}
            {toast.type === 'error' && <AlertCircle size={18} className="text-red-500" />}
            {toast.type === 'info' && <Info size={18} className="text-blue-500" />}
            <span className="text-sm font-medium">{toast.message}</span>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}