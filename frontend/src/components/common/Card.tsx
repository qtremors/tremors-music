// Reusable card component with glassmorphism
import { ReactNode } from 'react';
import { cn } from '../../lib/utils';

interface CardProps {
    children: ReactNode;
    className?: string;
    hover?: boolean;
    onClick?: () => void;
}

export function Card({ children, className, hover = false, onClick }: CardProps) {
    return (
        <div
            className={cn(
                'glass-panel p-6',
                hover && 'cursor-pointer hover:shadow-2xl hover:scale-[1.02] transition-all',
                onClick && 'cursor-pointer',
                className
            )}
            onClick={onClick}
        >
            {children}
        </div>
    );
}
