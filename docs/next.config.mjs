/* eslint-env node */

import Nextra from 'nextra'
import rehypeHighlightCodeBlock from '@mapbox/rehype-highlight-code-block'
import { fromLezer } from 'hast-util-from-lezer'
import { parser as rimuParser } from 'rimu-lezer'
import { toHtml } from 'hast-util-to-html'

const lezerParsers = {
  rimu: rimuParser,
}

const withNextra = Nextra({
  theme: 'nextra-theme-docs',
  themeConfig: './theme.config.js',
  unstable_staticImage: true,
  // codeHighlight: false,
  mdxOptions: {
    rehypePlugins: [
      [
        rehypeHighlightCodeBlock,
        {
          highlight,
        },
      ],
    ],
  },
})

const config = withNextra()

export default config

function highlight(code, lang) {
  const parser = lezerParsers[lang]
  if (parser == null) return null
  const tree = parser.parse(code)
  const element = fromLezer(code, tree)
  const html = toHtml(element)
  return html
    .split('\n')
    .map((line) => {
      if (line.length == 0) line = ' '
      return `<span class="line">${line}</span>`
    })
    .join('\n')
}
