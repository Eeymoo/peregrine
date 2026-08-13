/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{astro,html,js,ts,md}'],
  theme: {
    extend: {
      colors: {
        base: '#0B0E11',
        raised: '#14181D',
        ink: '#EDEDED',
        muted: '#8A9199',
        teal: '#14B8A6',
        hairline: 'rgba(255,255,255,0.08)',
      },
      fontFamily: {
        sans: ['"Inter Variable"', 'Inter', '"Noto Sans SC"', 'system-ui', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'ui-monospace', '"Noto Sans SC"', 'monospace'],
        serif: ['"Newsreader Variable"', '"Newsreader"', '"Noto Serif SC"', 'serif'],
      },
      borderRadius: {
        DEFAULT: '4px',
      },
      maxWidth: {
        prose: '65ch',
      },
      transitionTimingFunction: {
        lock: 'cubic-bezier(0.16, 1, 0.3, 1)',
      },
    },
  },
  plugins: [],
};
