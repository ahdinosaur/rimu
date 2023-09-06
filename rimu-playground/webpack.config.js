const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = {
  mode: "development",
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  experiments: {
    asyncWebAssembly: true,
    syncWebAssembly: true,
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [{ from: "index.html" }],
    }),
  ],
};
