import { ContextTracker, ExternalTokenizer } from '@lezer/lr'
import {
  endOfLine,
  endOfFile,
  indent,
  dedent,
  blankLineStart,
  listItemMarker,
} from './parser.terms'

const newline = 10,
  carriageReturn = 13,
  space = 32,
  hash = 35,
  dash = 45

function isLineBreak(ch) {
  return ch == newline || ch == carriageReturn
}

export const newlines = new ExternalTokenizer(
  (input, stack) => {
    let prev
    if (input.next < 0) {
      input.acceptToken(endOfFile)
    } else if (
      ((prev = input.peek(-1)) < 0 || isLineBreak(prev)) &&
      stack.canShift(blankLineStart)
    ) {
      let spaces = 0
      while (input.next == space) {
        input.advance()
        spaces++
      }
      if (input.next == newline || input.next == carriageReturn || input.next == hash)
        input.acceptToken(blankLineStart, -spaces)
    } else if (isLineBreak(input.next)) {
      input.acceptToken(endOfLine, 1)
    }
  },
  { contextual: true },
)

export const indentation = new ExternalTokenizer((input, stack) => {
  const cDepth = stack.context.depth
  const prev = input.peek(-1)
  if (prev === -1 || prev === newline || prev === carriageReturn) {
    let depth = 0
    let chars = 0
    for (;;) {
      if (input.next == space) depth++
      else break
      input.advance()
      chars++
    }
    if (
      depth != cDepth &&
      input.next != newline &&
      input.next != carriageReturn &&
      input.next != hash
    ) {
      if (depth < cDepth) input.acceptToken(dedent, -chars)
      else input.acceptToken(indent)
    }
  }
})

function peek(input, n) {
  let codes = []
  for (let i = -1; i < n; i++) {
    codes.push(JSON.stringify(String.fromCharCode(input.peek(i))))
  }
  return codes.join(', ')
}

export const listItemMarkers = new ExternalTokenizer((input, _stack) => {
  if (input.next === dash) {
    let spacesAfterDash = 0
    while (input.advance() === space) {
      spacesAfterDash++
    }
    if (spacesAfterDash > 0) {
      input.acceptToken(listItemMarker)
    }
  }
})

class IndentLevel {
  constructor(parent, depth, type) {
    this.parent = parent
    this.depth = depth
    this.type = type
    this.hash = (parent ? (parent.hash + parent.hash) << 8 : 0) + depth + (depth << 4)
  }
}

export const trackIndent = new ContextTracker({
  start: new IndentLevel(null, 0, 'base'),
  shift(context, term, stack, input) {
    if (term == indent) {
      const depth = stack.pos - input.pos
      return new IndentLevel(context, depth, 'indent')
    }
    if (term == dedent) {
      return context.parent
    }
    if (term == listItemMarker) {
      const depth = context.depth + stack.pos - input.pos
      return new IndentLevel(context, depth, 'list-item')
    }
    return context
  },
  hash(context) {
    return context.hash
  },
})
