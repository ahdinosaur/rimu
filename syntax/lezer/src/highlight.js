// with help from
//
// - https://github.com/lezer-parser/javascript/blob/main/src/highlight.js

import { styleTags, tags as t } from '@lezer/highlight'

export const highlight = styleTags({
  'if then else': t.controlKeyword,
  'let in': t.definitionKeyword,

  null: t.null,
  'true false': t.bool,
  String: t.string,
  Number: t.number,

  ArithmeticOp: t.arithmeticOperator,
  LogicOp: t.logicOperator,
  // BitOp: t.bitwiseOperator,
  CompareOp: t.compareOperator,
  FatArrow: t.function(t.punctuation),

  Identifier: t.variableName,
  LetKey: t.definition(t.variableName),
  GetIdentifier: t.propertyName,
  ObjectKey: t.definition(t.propertyName),
  FunctionParam: t.definition(t.variableName),

  LineComment: t.lineComment,

  '( )': t.paren,
  '[ ]': t.squareBracket,
  '{ }': t.brace,
  ',': t.separator,
  '.': t.derefOperator,
})
