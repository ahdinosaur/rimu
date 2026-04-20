import nextCoreWebVitals from 'eslint-config-next/core-web-vitals'

export default [
  {
    ignores: ['.next/**', 'node_modules/**', 'pkg/**', 'wasm/pkg/**'],
  },
  ...nextCoreWebVitals,
]
