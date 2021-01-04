const path = require('path');
// const webpack = require('webpack');
// const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './index.js',
    output: {
        path: path.resolve(__dirname, 'static'),
        filename: 'bundle.js',
        // ensures we load all resources from root of site
        publicPath: "/",
    },
    mode: 'development',
    resolve: {
        modules: [path.resolve(__dirname, "node_modules")]
    },
    target: "web"
};
