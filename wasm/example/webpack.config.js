const HtmlWebPackPlugin = require("html-webpack-plugin");
const htmlPlugin = new HtmlWebPackPlugin({
  template: "./src/index.html",
  filename: "./index.html",
});
module.exports = {
  mode: "development",
  devServer: {
    port: 18080,
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader",
        },
      },
      {
        test: /\.css$/,
        use: ["style-loader", "css-loader"],
      },
      {
        test: /\.tsx?$/,
        use: "ts-loader",
        exclude: /node_modules/,
      },
      {
        test: /\.wasm$/,
        type: "asset/inline",
      },
    ],
  },
  plugins: [htmlPlugin],
  resolve: {
    fallback: {
      os: false,
      url: false,
      path: false,
      fs: false,
      path: false,
      util: false,
      vm: false,
      tty: false,
      http: false,
      https: false,
    },
    extensions: [".tsx", ".ts", ".js"],
  },
};
