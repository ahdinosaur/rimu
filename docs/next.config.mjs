/* eslint-env node */

import { getHighlighter, BUNDLED_LANGUAGES } from 'shiki'
import Nextra from 'nextra'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))

const withNextra = Nextra({
  theme: 'nextra-theme-docs',
  themeConfig: './theme.config.js',
  unstable_staticImage: true,
  mdxOptions: {
    rehypePrettyCodeOptions: {
      getHighlighter: (options) =>
        getHighlighter({
          ...options,
          langs: [
            ...BUNDLED_LANGUAGES,
            {
              id: 'rimu',
              scopeName: 'source.rimu',
              aliases: [],
              path: join(__dirname, '../syntax/textmate/rimu.tmLanguage.json'),
            },
          ],
        }),
    },
  },
})

const config = withNextra()

export default config
