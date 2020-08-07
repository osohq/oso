const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const dist = path.resolve(__dirname, 'dist');
const rust = path.resolve(__dirname, '../../polar-wasm-api');

module.exports = {
  mode: 'development',
  target: 'node',
  entry: { index: './src/index.ts' },
  output: { path: dist, filename: '[name].js' },
  devServer: { contentBase: dist },
  module: {
    rules: [{ test: /\.ts$/, use: 'ts-loader', exclude: /node_modules/ }],
  },
  plugins: [new WasmPackPlugin({ crateDirectory: rust, withTypeScript: true })],
  resolve: {
    extensions: ['.ts', '.js', '.wasm'],
  },
};
