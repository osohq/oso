const path = require('path');
// const webpack = require('webpack');
// const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: './index.js',
  output: {
    path: path.resolve(__dirname, 'static'),
    publicPath: '/',
    filename: 'bundle.js'
  },
  mode: 'development',
  resolve: {
    modules: [path.resolve(__dirname, 'node_modules')]
  },
  module: {
    rules: [{
        test: /\.css$/i,
        use: ['style-loader', 'css-loader']
      },
      {
        test: /\.ttf$/,
        use: ['file-loader']
      },
      {
        test: /\.handlebars$/,
        loader: "handlebars-loader"
      }
    ]
  },
  target: 'web'
};
