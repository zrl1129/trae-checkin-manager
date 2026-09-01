/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        bg: {
          primary: "#0a0a0b",
          secondary: "#131316",
          tertiary: "#1a1a1f",
        },
        accent: {
          blue: "#3b82f6",
          green: "#10b981",
          orange: "#f59e0b",
          red: "#ef4444",
          purple: "#8b5cf6",
        },
      },
      fontFamily: {
        mono: ["JetBrains Mono", "Geist Mono", "monospace"],
        sans: ["Inter", "Geist", "system-ui", "sans-serif"],
      },
    },
  },
  plugins: [],
};
