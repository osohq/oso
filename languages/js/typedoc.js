module.exports = {
  tsconfig: './tsconfig.build.json',
  entryPoints: ['./src'],
  out: 'docs',
  exclude: [
    './test/*',
    './src/**/*.test.ts',
    './src/**/polar_wasm_api*',
    './src/index.ts',
    './src/helpers.ts',
    './src/messages.ts',
  ],
  excludePrivate: true,
  excludeProtected: true,
  excludeExternals: true,
  externalPattern: '**/node_modules/*',
  theme: 'default',
  readme: '../../README.md',
  gaID: 'UA-139858805-1',
  hideGenerator: true,
  validation: {
    invalidLink: true,
  },
};
