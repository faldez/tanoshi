const colors = require('tailwindcss/colors')

module.exports = {
    theme: {
        extend: {
            colors: {
                'accent': '#991B1B',
                'accent-lighter': '#B91C1C',
                'accent-darker': '#7F1D1D',
            },
            height: {
                page: 'calc(100vw * 1.59)',
                '1/2': '50%',
            },
            spacing: {
                '7/5': '141.5094339622642%',
                'safe-bottom': 'env(safe-area-inset-bottom)',
                'safe-top': 'env(safe-area-inset-top)'
            },
        },
        colors: {
            transparent: 'transparent',
            current: 'currentColor',
            black: colors.black,
            white: colors.white,
            gray: colors.trueGray,
            red: colors.red,
            yellow: colors.amber,
            blue: colors.blue
        },
        container: {
            center: true,
        },
        minWidth: {
            '0': '0',
            '1/4': '25%',
            '1/2': '50%',
            '3/4': '75%',
            'full': '100%',
        }
    },
    variants: {
        backgroundColor: ['dark', 'responsive', 'hover', 'focus', 'disabled'],
        textColor: ['dark', 'responsive', 'hover', 'focus', 'disabled'],
    },
    plugins: [],
    darkMode: 'class'
}
