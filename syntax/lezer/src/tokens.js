import { ContextTracker, ExternalTokenizer } from '@lezer/lr'
import { indent, dedent, blankLineStart } from './parser.terms'
const newline = 10,
  carriageReturn = 13,
  space = 32,
  hash = 35,
  dash = 45

function isLineBreak(ch) {
  return ch == newline || ch == carriageReturn
}

export const indentation = new ExternalTokenizer((input, stack) => {
  let prev = input.peek(-1)
  if (prev != -1 && prev != newline && prev != carriageReturn) return
  let spaces = 0
  while (input.next == space) {
    input.advance()
    spaces++
  }
  if ((isLineBreak(input.next) || input.next == hash) && stack.canShift(blankLineStart)) {
    input.acceptToken(blankLineStart, -spaces)
  } else if (spaces > stack.context.depth) {
    input.acceptToken(indent)
  } else if (spaces < stack.context.depth) {
    input.acceptToken(dedent, -spaces)
  }
})

class IndentLevel {
  constructor(parent, depth) {
    this.parent = parent
    // -1 means this is not an actual indent level but a set of brackets
    this.depth = depth
    this.hash = (parent ? (parent.hash + parent.hash) << 8 : 0) + depth + (depth << 4)
  }
}

export const trackIndent = new ContextTracker({
  start: new IndentLevel(null, 0),
  shift(context, term, stack, input) {
    if (term == indent) return new IndentLevel(context, stack.pos - input.pos)
    if (term == dedent) return context.parent
    return context
  },
  hash(context) {
    return context.hash
  },
})
