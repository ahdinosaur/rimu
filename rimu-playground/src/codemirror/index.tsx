import { EditorState } from '@codemirror/state'
import { basicSetup } from 'codemirror'
import { EditorView, keymap } from '@codemirror/view'
import * as yamlMode from '@codemirror/legacy-modes/mode/yaml'
import { StreamLanguage, LanguageSupport } from '@codemirror/language'
import { indentWithTab } from '@codemirror/commands'
import { catppuccin, Variant } from 'codemirror-theme-catppuccin'
import { variants } from '@catppuccin/palette'

import { createDiagnostics, createDiagnosticGutter } from './diagnostics'
import { createIdler } from './idle'
import { createDiagnosticTheme, createDiagnosticGutterTheme } from './theme'

export type CodeMirrorOptions = {
  parent: HTMLDivElement
  theme: Variant
  initialCode: string
  setCode: (code: string) => void
}

export function CodeMirror(options: CodeMirrorOptions) {
  const { parent, theme, initialCode, setCode } = options

  const yaml = new LanguageSupport(StreamLanguage.define(yamlMode.yaml))

  const idler = createIdler(
    (view) => {
      const code = view.state.sliceDoc()
      setCode(code)
    },
    {
      delay: 50,
    },
  )

  const palette = variants[theme]

  const startState = EditorState.create({
    doc: initialCode,
    extensions: [
      basicSetup,
      keymap.of([indentWithTab]),
      catppuccin(theme),
      yaml,
      idler,
      createDiagnostics(),
      createDiagnosticTheme(palette),
      createDiagnosticGutter(),
      createDiagnosticGutterTheme(palette),
    ],
  })

  const view = new EditorView({ state: startState, parent })

  return view
}
