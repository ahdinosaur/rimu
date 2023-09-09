const { resolve } = require('path')
const { readdirSync } = require('fs')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')

/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack(config) {
    config.experiments.asyncWebAssembly = true

    config.plugins.push(
      new WasmPackPlugin({
        crateDirectory: resolve(__dirname, 'wasm'),

        /*
        watchDirectories: [
          ...readdirSync(resolve(__dirname, '..')).filter((dir) => dir.startsWith('rimu-')),
        ],
        */
      }),
    )

    return config
  },
}

module.exports = nextConfig
