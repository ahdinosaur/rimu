const { resolve } = require('path')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')

/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack(config) {
    config.plugins.push(
      new WasmPackPlugin({
        crateDirectory: resolve(__dirname, 'wasm'),
        extraArgs: '--target web',
      }),
    )

    config.module.rules.push({ test: /\.rimu$/, type: 'asset/source' })

    return config
  },
}

module.exports = nextConfig
