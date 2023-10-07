/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./public/templates/*.html"],
    theme: {
        extend: {
            fontFamily: {
                bird: ["Birdman", "sans-serif"],
                sanset: ["Sanset", "sans-serif"],
                dune: ["Dune", "sans-serif"],
                cascade: ["Cascadia", "sans-serif"],
            },
        },
    },
    plugins: [require("@tailwindcss/forms"),],
}

