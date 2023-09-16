/* eslint-env node */

import Nextra from 'nextra'

const withNextra = Nextra({
  theme: 'nextra-theme-docs',
  themeConfig: './theme.config.js',
  unstable_staticImage: true,
})

const config = withNextra()

export default config
