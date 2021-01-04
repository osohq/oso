const themeDir = __dirname + '/../../';

const purgecss = require('@fullhuman/postcss-purgecss')({

    // Specify the paths to all of the template files in your project
    content: [
        themeDir + 'layouts/**/*.html',
        themeDir + 'content/**/*.html',
        'layouts/**/*.html',
        'content/**/*.html',
    ],

    whitelistPatternsChildren: [/^prose$/],

    // This is the function used to extract class names from your templates
    defaultExtractor: content => {
        // Capture as liberally as possible, including things like `h-(screen-1.5)`
        const broadMatches = content.match(/[^<>"'`\s]*[^<>"'`\s:]/g) || []

        // Capture classes within other delimiters like .block(class="w-1/2") in Pug
        const innerMatches = content.match(/[^<>"'`\s.()]*[^<>"'`\s.():]/g) || []

        return broadMatches.concat(innerMatches)
    }
})

module.exports = {
    plugins: [
        require('postcss-import')({
            path: [themeDir]
        }),
        require('tailwindcss')(themeDir + 'assets/css/tailwind.config.js'),
        require('autoprefixer')({
            path: [themeDir]
        }),
        ...(process.env.HUGO_ENVIRONMENT === 'production' ? [purgecss] : [])
    ]
}
