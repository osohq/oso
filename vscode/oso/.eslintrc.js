module.exports = {
  root: true,
  extends: ['./node_modules/gts/'],
  ignorePatterns: [
    'node_modules', // Self-explanatory.
    'client/out', // Don't lint built client library.
    'server/out', // Don't lint built server library.
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
          // TODO(gj): remove server when moving from toy TS server -> Rust.
          './server/tsconfig.json',
        ],
      },
    },
  ],
};
