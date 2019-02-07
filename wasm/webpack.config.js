const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const WEBPACK_MODE =
  (typeof process.env.WEBPACK_MODE === 'undefined') ? "development" : process.env.WEBPACK_MODE;
const OUTPUT_DIR=
  (typeof process.env.OUTPUT_DIR === 'undefined') ? path.resolve(__dirname, "dist") : process.env.OUTPUT_DIR;

module.exports = {
    mode: WEBPACK_MODE, // values can be "development" or "production"
    entry: "./js/index.js",
    devtool: "source-map",
    output: {
        path: OUTPUT_DIR,
        filename: "bundle.js",
        webassemblyModuleFilename: "app.wasm",
    },
    plugins: [
        new CopyWebpackPlugin([{ from: "static_web" }]),
    ],
};
