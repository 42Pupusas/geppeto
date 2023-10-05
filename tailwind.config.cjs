/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./public/templates/*.html'],
  theme: {
    extend: {},
  },
  plugins: [require('@tailwindcss/forms')],
}

