// Icon button component for actions
import { ButtonHTMLAttributes, ReactNode } from 'react';
import { cn } from '../../lib/utils';

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    icon: ReactNode;
    tooltip?: string;
    variant?: 'default' | 'accent' | 'ghost';
    size?: 'sm' | 'md' | 'lg';
}

export function IconButton({
    icon,
    tooltip,
    variant = 'default',
    size = 'md',
    className,
    ...props
}: IconButtonProps) {
    const sizes = {
        sm: 'w-8 h-8',
        md: 'w-10 h-10',
        lg: 'w-12 h-12'
    };

    const variants = {
        default: 'text-apple-text hover:bg-gray-100 dark:hover:bg-white/10',
        accent: 'text-white bg-apple-accent hover:opacity-90',
        ghost: 'text-apple-subtext hover:text-apple-text hover:bg-gray-100 dark:hover:bg-white/10'
    };

    return (
        <button
            className={cn(
                'inline-flex items-center justify-center rounded-full transition-all focus:outline-none focus:ring-2 focus:ring-apple-accent/50 disabled:opacity-50',
                sizes[size],
                variants[variant],
                className
            )}
            title={tooltip}
            {...props}
        >
            {icon}
        </button>
    );
}
