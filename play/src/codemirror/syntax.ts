// @ts-ignore
import { parser, highlight } from 'rimu-lezer'
import { LRLanguage, LanguageSupport } from '@codemirror/language'

const languageParser = parser.configure({
  props: [
    highlight,
    /*
    indentNodeProp.add({
      Application: (context) => context.column(context.node.from) + context.unit,
    }),
    foldNodeProp.add({
      Application: foldInside,
    }),
    */
  ],
})

const language = LRLanguage.define({
  parser: languageParser,
  languageData: {
    commentTokens: { line: '#' },
  },
})

export function syntax() {
  return new LanguageSupport(language)
}
