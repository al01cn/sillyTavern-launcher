/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{vue,js,ts,jsx,tsx}",
    ],
    theme: {
        extend: {
            fontFamily: {
                sans: ["Inter",
                    "SF Pro Display",
                    "BlinkMacSystemFont",
                    "Segoe UI",
                    "PingFang SC",
                    "Hiragino Sans GB",
                    "Microsoft YaHei",
                    "Helvetica Neue",
                    "Arial",],
                mono: ["Inter",
                    "SF Pro Display",
                    "BlinkMacSystemFont",
                    "Segoe UI",
                    "PingFang SC",
                    "Hiragino Sans GB",
                    "Microsoft YaHei",
                    "Helvetica Neue",
                    "Arial"],
            },
            colors: {
                primary: { DEFAULT: '#4DB7FF', hover: '#66C5FF', active: '#1FA4FF', light: '#E0F2FE' },
                success: '#4ADE80',
                warning: '#FFB347',
                error: '#FF6B6B',
                surface: '#F8FAFC',
                card: '#FFFFFF',
                border: '#E2E8F0'
            },
            boxShadow: {
                'soft': '0 8px 30px rgba(0,0,0,0.04)',
                'float': '0 10px 40px rgba(77, 183, 255, 0.2)',
                'menu': '0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1)',
                'modal': '0 20px 50px -12px rgba(0, 0, 0, 0.15)'
            }
        }
    },
    plugins: [],
}