/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class', // Enable dark mode via class
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
      },
      colors: {
        apple: {
          gray: 'rgb(var(--bg-color) / <alpha-value>)',
          text: 'rgb(var(--text-color) / <alpha-value>)',
          subtext: 'rgb(var(--subtext-color) / <alpha-value>)',
          accent: 'rgb(var(--accent-color) / <alpha-value>)',
        }
      },
    },
  },
  plugins: [],
}