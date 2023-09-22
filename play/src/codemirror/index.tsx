import { EditorState, Compartment } from '@codemirror/state'
import { basicSetup } from 'codemirror'
import { EditorView, keymap } from '@codemirror/view'
import { indentWithTab } from '@codemirror/commands'
import { catppuccin, Variant } from 'codemirror-theme-catppuccin'
import { variants } from '@catppuccin/palette'

import { syntax } from './syntax'
import { createDiagnostics, createDiagnosticGutter } from './diagnostics'
import { createIdler } from './idle'
import { createDiagnosticTheme, createDiagnosticGutterTheme } from './theme'

export type CodeMirrorOptions = {
  theme: Variant
  code: string
  setCode: (code: string) => void
  setState: (state: EditorState) => void
}

const themeCompartment = new Compartment()

export function CodeMirror(options: CodeMirrorOptions) {
  const { theme, code, setCode, setState } = options

  const idler = createIdler(
    (view) => {
      const code = view.state.sliceDoc()
      setCode(code)
    },
    {
      delay: 50,
    },
  )

  return EditorState.create({
    doc: code,
    extensions: [
      basicSetup,
      EditorView.updateListener.of((ev) => {
        setState(ev.state)
      }),
      keymap.of([indentWithTab]),
      themeCompartment.of([
        catppuccin(theme),
        createDiagnosticTheme(variants[theme]),
        createDiagnosticGutterTheme(variants[theme]),
      ]),
      syntax(),
      idler,
      createDiagnostics(),
      createDiagnosticGutter(),
    ],
  })
}

export function updateTheme(view: EditorView, theme: Variant) {
  view.dispatch({
    effects: [
      themeCompartment.reconfigure([
        catppuccin(theme),
        createDiagnosticTheme(variants[theme]),
        createDiagnosticGutterTheme(variants[theme]),
      ]),
    ],
  })
}

export function updateCode(view: EditorView, code: string) {
  view.dispatch({
    changes: {
      from: 0,
      to: view.state.doc.length,
      insert: code,
    },
  })
}
