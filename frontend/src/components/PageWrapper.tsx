import { motion } from 'framer-motion';
import { pageVariants } from '../lib/animations';

interface PageWrapperProps {
    children: React.ReactNode;
    className?: string;
}

export function PageWrapper({ children, className = '' }: PageWrapperProps) {
    return (
        <motion.div
            initial="initial"
            animate="animate"
            exit="exit"
            variants={pageVariants}
            className={`h-full ${className}`}
        >
            {children}
        </motion.div>
    );
}
