import React from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';

interface ErrorBoundaryState {
    hasError: boolean;
    error: Error | null;
    errorInfo: React.ErrorInfo | null;
}

interface ErrorBoundaryProps {
    children: React.ReactNode;
    fallback?: React.ReactNode;
    onReset?: () => void;
}

/**
 * React Error Boundary component for catching and handling errors gracefully.
 * Provides a fallback UI when errors occur in child components.
 */
export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
    constructor(props: ErrorBoundaryProps) {
        super(props);
        this.state = { hasError: false, error: null, errorInfo: null };
    }

    static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
        console.error('ErrorBoundary caught an error:', error, errorInfo);
        this.setState({ errorInfo });
    }

    handleReset = () => {
        this.setState({ hasError: false, error: null, errorInfo: null });
        this.props.onReset?.();
    };

    handleGoHome = () => {
        window.location.href = '/';
    };

    render() {
        if (this.state.hasError) {
            // Custom fallback provided
            if (this.props.fallback) {
                return this.props.fallback;
            }

            // Default error UI
            return (
                <ErrorFallback
                    error={this.state.error}
                    onRetry={this.handleReset}
                    onGoHome={this.handleGoHome}
                />
            );
        }

        return this.props.children;
    }
}

/**
 * Fallback UI component displayed when an error is caught.
 */
interface ErrorFallbackProps {
    error: Error | null;
    onRetry?: () => void;
    onGoHome?: () => void;
    title?: string;
    message?: string;
}

export function ErrorFallback({
    error,
    onRetry,
    onGoHome,
    title = 'Something went wrong',
    message = 'An unexpected error occurred. Please try again.',
}: ErrorFallbackProps) {
    return (
        <div className="min-h-[400px] flex flex-col items-center justify-center p-8 text-center">
            <div className="w-16 h-16 rounded-full bg-red-100 dark:bg-red-900/20 flex items-center justify-center mb-6">
                <AlertTriangle size={32} className="text-red-500" />
            </div>

            <h2 className="text-xl font-semibold text-apple-text mb-2">{title}</h2>
            <p className="text-apple-subtext max-w-md mb-6">{message}</p>

            {error && (
                <div className="bg-gray-100 dark:bg-white/5 rounded-lg p-4 max-w-lg w-full mb-6 text-left">
                    <p className="text-xs font-mono text-red-500 dark:text-red-400 break-all">
                        {error.message}
                    </p>
                </div>
            )}

            <div className="flex gap-3">
                {onRetry && (
                    <button
                        onClick={onRetry}
                        className="flex items-center gap-2 px-4 py-2 bg-apple-accent text-white rounded-lg hover:bg-apple-accent/90 transition font-medium"
                    >
                        <RefreshCw size={16} />
                        Try Again
                    </button>
                )}
                {onGoHome && (
                    <button
                        onClick={onGoHome}
                        className="flex items-center gap-2 px-4 py-2 bg-gray-200 dark:bg-white/10 text-apple-text rounded-lg hover:bg-gray-300 dark:hover:bg-white/15 transition font-medium"
                    >
                        <Home size={16} />
                        Go Home
                    </button>
                )}
            </div>
        </div>
    );
}

/**
 * Hook-based error boundary wrapper for functional components.
 * Use this with React Query's error handling.
 */
interface QueryErrorFallbackProps {
    error: Error;
    refetch?: () => void;
}

export function QueryErrorFallback({ error, refetch }: QueryErrorFallbackProps) {
    return (
        <ErrorFallback
            error={error}
            title="Failed to load data"
            message="We couldn't load this content. Please check your connection and try again."
            onRetry={refetch}
        />
    );
}

/**
 * Compact error message for inline use.
 */
interface InlineErrorProps {
    message?: string;
    onRetry?: () => void;
}

export function InlineError({
    message = 'Failed to load',
    onRetry
}: InlineErrorProps) {
    return (
        <div className="flex items-center gap-2 p-3 bg-red-50 dark:bg-red-900/10 rounded-lg text-red-600 dark:text-red-400">
            <AlertTriangle size={16} />
            <span className="text-sm flex-1">{message}</span>
            {onRetry && (
                <button
                    onClick={onRetry}
                    className="p-1 hover:bg-red-100 dark:hover:bg-red-900/20 rounded transition"
                >
                    <RefreshCw size={14} />
                </button>
            )}
        </div>
    );
}
