// Animation variants for Framer Motion
export const pageVariants = {
    initial: {
        opacity: 0,
        y: 20,
    },
    animate: {
        opacity: 1,
        y: 0,
        transition: {
            duration: 0.3,
            ease: [0.4, 0, 0.2, 1] as const, // easeOut
        },
    },
    exit: {
        opacity: 0,
        y: -20,
        transition: {
            duration: 0.2,
            ease: [0.4, 0, 1, 1] as const, // easeIn
        },
    },
};

export const staggerContainer = {
    animate: {
        transition: {
            staggerChildren: 0.05,
        },
    },
};

export const staggerItem = {
    initial: { opacity: 0, y: 10 },
    animate: {
        opacity: 1,
        y: 0,
        transition: {
            duration: 0.3,
        },
    },
};

export const scaleOnHover = {
    rest: { scale: 1 },
    hover: {
        scale: 1.05,
        transition: {
            duration: 0.2,
            ease: [0.4, 0, 0.2, 1] as const,
        },
    },
};

export const fadeIn = {
    initial: { opacity: 0 },
    animate: {
        opacity: 1,
        transition: { duration: 0.3 },
    },
    exit: {
        opacity: 0,
        transition: { duration: 0.2 },
    },
};

export const slideUp = {
    initial: { y: '100%' },
    animate: {
        y: 0,
        transition: {
            duration: 0.3,
            ease: [0.4, 0, 0.2, 1] as const,
        },
    },
    exit: {
        y: '100%',
        transition: {
            duration: 0.2,
            ease: [0.4, 0, 1, 1] as const,
        },
    },
};

export const slideInFromRight = {
    initial: { x: '100%' },
    animate: {
        x: 0,
        transition: {
            duration: 0.3,
            ease: [0.4, 0, 0.2, 1] as const,
        },
    },
    exit: {
        x: '100%',
        transition: {
            duration: 0.2,
            ease: [0.4, 0, 1, 1] as const,
        },
    },
};
