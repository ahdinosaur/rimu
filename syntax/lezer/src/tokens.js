import { ContextTracker, ExternalTokenizer } from '@lezer/lr'
import { indent, dedent, blankLineStart, listItemMarker } from './parser.terms'

const newline = 10,
  carriageReturn = 13,
  space = 32,
  hash = 35,
  dash = 45

function isLineBreak(ch) {
  return ch == newline || ch == carriageReturn
}

export const lines = new ExternalTokenizer(
  (input, stack) => {
    // if at end of file, dedent remaining indents
    if (input.next == -1 && stack.context.depth > 0) {
      console.log('dedent')
      input.acceptToken(dedent)
      return
    }

    let prev = input.peek(-1)
    if (prev != -1 && prev != newline && prev != carriageReturn) return

    let spaces = 0
    while (input.next == space) {
      input.advance()
      spaces++
    }

    // empty line
    if ((isLineBreak(input.next) || input.next == hash) && stack.canShift(blankLineStart)) {
      input.acceptToken(blankLineStart, -spaces)
    }

    // indent
    else if (spaces > stack.context.depth) {
      input.acceptToken(indent)
    }

    // dedents
    let context = stack.context
    while (spaces < context.depth) {
      input.acceptToken(dedent, -spaces)
      context = context.parent
    }

    // handle list item markers with a special token
    while (input.next == dash) {
      if (input.peek(1) != space) break

      while (input.advance() == space) {}

      input.acceptToken(listItemMarker)
    }
  },
  {
    contextual: true,
  },
)

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
    console.log('input', term, JSON.stringify(String.fromCharCode(term)))
    console.log('indent', term === indent)
    console.log('dedent', term === dedent)
    console.log('listItemMarker', term === listItemMarker)
    console.log('stack.pos', stack.pos)
    console.log('input.pos', input.pos)
    if (term == indent) return new IndentLevel(context, stack.pos - input.pos)
    if (term == dedent) return context.parent
    if (term == listItemMarker) return new IndentLevel(context, stack.pos - input.pos)
    return context
  },
  hash(context) {
    return context.hash
  },
})
