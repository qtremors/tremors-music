// Common reusable button component with variants
import { ButtonHTMLAttributes, ReactNode } from 'react';
import { cn } from '../../lib/utils';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    children: ReactNode;
    isLoading?: boolean;
}

export function Button({
    variant = 'primary',
    size = 'md',
    children,
    className,
    isLoading,
    disabled,
    ...props
}: ButtonProps) {
    const baseStyles = 'inline-flex items-center justify-center gap-2 font-medium rounded-lg transition-all focus:outline-none focus:ring-2 focus:ring-apple-accent/50 disabled:opacity-50 disabled:cursor-not-allowed';

    const variants = {
        primary: 'bg-apple-accent text-white hover:opacity-90 shadow-lg shadow-apple-accent/20',
        secondary: 'bg-gray-100 dark:bg-white/10 text-apple-text hover:bg-gray-200 dark:hover:bg-white/20',
        ghost: 'text-apple-text hover:bg-gray-100 dark:hover:bg-white/10',
        danger: 'bg-red-500 text-white hover:bg-red-600 shadow-lg shadow-red-500/20'
    };

    const sizes = {
        sm: 'px-3 py-1.5 text-sm',
        md: 'px-4 py-2 text-base',
        lg: 'px-6 py-3 text-lg'
    };

    return (
        <button
            className={cn(baseStyles, variants[variant], sizes[size], className)}
            disabled={disabled || isLoading}
            {...props}
        >
            {isLoading ? (
                <>
                    <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
                    Loading...
                </>
            ) : children}
        </button>
    );
}
