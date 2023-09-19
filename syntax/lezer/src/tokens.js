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

export const listItemMarkers = new ExternalTokenizer((input, _stack) => {
  // handle list item markers with a special token
  while (input.next == dash) {
    if (input.peek(1) != space) break
    while (input.advance() == space) {}
    input.acceptToken(listItemMarker)
  }
})

export const indentation = new ExternalTokenizer((input, stack) => {
  const cDepth = stack.context.depth
  if (cDepth < 0) return
  const prev = input.peek(-1)
  if (prev == newline || prev == carriageReturn) {
    const depth = 0
    const chars = 0
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

class IndentLevel {
  constructor(parent, depth) {
    this.parent = parent
    this.depth = depth
    this.hash = (parent ? (parent.hash + parent.hash) << 8 : 0) + depth + (depth << 4)
  }
}

export const trackIndent = new ContextTracker({
  start: new IndentLevel(null, 0),
  shift(context, term, stack, input) {
    if (term == indent) return new IndentLevel(context, stack.pos - input.pos)
    if (term == dedent) return context.parent
    if (term == listItemMarker) return new IndentLevel(context, stack.pos - input.pos)
    return context
  },
  hash(context) {
    return context.hash
  },
})
