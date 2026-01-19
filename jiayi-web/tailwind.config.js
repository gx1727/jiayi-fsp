/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        brand: {
          blue: "#1e40af",
          purple: "#7c3aed",
          gray: "#6b7280",
        },
      },
      boxShadow: {
        soft: "0 10px 30px rgba(30,64,175,0.08)",
      },
    },
  },
  plugins: [],
};

