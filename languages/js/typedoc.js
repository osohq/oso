module.exports = {
  entryPoints: ['./src'],
  out: 'docs',
  exclude: [
    './test/*',
    './src/**/*.test.ts',
    './src/**/polar_wasm_api*',
    './src/index.ts',
  ],
  excludePrivate: true,
  excludeProtected: true,
  excludeExternals: true,
  externalPattern: '**/node_modules/*',
  theme: 'default',
  readme: '../../README.md',
  gaID: 'UA-139858805-1',
  hideGenerator: true,
  listInvalidSymbolLinks: true,
};
