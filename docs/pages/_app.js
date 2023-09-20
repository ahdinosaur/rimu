import { variants } from '@catppuccin/palette'
import { StyleModule } from 'style-mod'
import { tags as t, tagHighlighter } from '@lezer/highlight'

import '../styles.css'
import { useEffect } from 'react'

export default function MyApp({ Component, pageProps }) {
  useEffect(() => {
    const tagStyles = catppuccinHighlightStyle('mocha')
    const style = tagHighlighter(tagStyles, { all: 'hellllooooo' })
    console.log(style)
  }, [])

  return <Component {...pageProps} />
}

/// The highlighting style for code in the Catppuccin theme.
export function catppuccinHighlightStyle(variant) {
  const palette = variants[variant]

  return [
    // keyword
    { tag: [t.keyword, t.bool], color: palette.mauve.hex },
    // string
    {
      tag: [t.processingInstruction, t.string, t.inserted],
      color: palette.green.hex,
    },
    // escape sequences
    {
      tag: [t.escape, t.regexp, t.special(t.string)],
      color: palette.pink.hex,
    },
    // comments
    { tag: [t.comment], color: palette.overlay0.hex },
    // constants, numbers
    {
      tag: [t.number, t.atom, t.color, t.constant(t.name), t.standard(t.name)],
      color: palette.peach.hex,
    },
    // operators
    { tag: [t.operator, t.operatorKeyword], color: palette.sky.hex },
    // braces, delimiters
    { tag: [t.deleted, t.character, t.separator], color: palette.overlay2.hex },
    // methods, functions
    { tag: [t.function(t.variableName), t.labelName], color: palette.blue.hex },
    // parameters
    {
      tag: [t.propertyName, t.macroName],
      color: palette.maroon.hex,
    },
    // local variables
    {
      tag: [t.special(t.variableName), t.definition(t.name), t.name],
      color: palette.teal.hex,
    },
    // built-ins

    // classes, metadata
    {
      tag: [t.meta, t.typeName, t.className, t.annotation, t.modifier, t.self, t.namespace],
      color: palette.yellow.hex,
    },
    // link
    {
      tag: [t.url, t.link],
      color: palette.sky.hex,
      textDecoration: 'underline',
    },
    // other
    { tag: t.strong, fontWeight: 'bold' },
    { tag: t.emphasis, fontStyle: 'italic' },
    { tag: t.strikethrough, textDecoration: 'line-through' },
    { tag: t.heading, fontWeight: 'bold', color: palette.lavender.hex },
    { tag: t.invalid, color: palette.red.hex },
  ]
}
