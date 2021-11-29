module.exports = {
  root: true,
  extends: ['./node_modules/gts/'],
  ignorePatterns: [
    'node_modules', // Self-explanatory.
    'out', // Don't lint built library.
  ],
  overrides: [
    {
      files: ['**/*.ts', '**/*.tsx'],
      extends: [
        'plugin:@typescript-eslint/recommended',
        'plugin:@typescript-eslint/recommended-requiring-type-checking',
      ],
      parserOptions: {
        tsconfigRootDir: __dirname,
        project: [
          './client/tsconfig.json',
          './server/tsconfig.json',
          './test/tsconfig.json',
        ],
      },
    },
  ],
};
