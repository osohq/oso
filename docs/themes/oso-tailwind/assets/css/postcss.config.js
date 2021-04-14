const themeDir = __dirname + '/../../';

// https://github.com/gohugoio/hugoDocs/blob/399c74acd69aa7d17e72c03942a84a66d4857f31/content/en/hugo-pipes/postprocess.md#css-purging-with-postcss
const purgecss = require('@fullhuman/postcss-purgecss')({
  content: ['./hugo_stats.json'],
  safelist: ["polar-code-in-here"],
  defaultExtractor: (content) => {
    let els = JSON.parse(content).htmlElements;
    return els.tags.concat(els.classes, els.ids);
  },
});

module.exports = {
  plugins: [
    require('postcss-import')({
      path: [themeDir],
    }),
    require('tailwindcss')(themeDir + 'assets/css/tailwind.config.js'),
    require('autoprefixer')({
      path: [themeDir],
    }),
    ...(process.env.HUGO_PURGECSS !== 'off' ? [purgecss] : [])
  ],
};
