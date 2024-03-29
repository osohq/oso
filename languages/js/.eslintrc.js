module.exports = {
  root: true,
  extends: ['./node_modules/gts/'],
  ignorePatterns: [
    'node_modules', // Self-explanatory.
    'dist', // Don't lint built library.
    'docs', // Don't lint files generated by TypeDoc.
    'src/polar_wasm_api*', // Don't lint files generated by wasm-pack.
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
        project: ['./tsconfig.json'],
      },
    },
  ],
};
