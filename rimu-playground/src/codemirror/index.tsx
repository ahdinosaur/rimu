import { EditorState } from '@codemirror/state'
import { basicSetup } from 'codemirror'
import { EditorView, keymap } from '@codemirror/view'
import * as yamlMode from '@codemirror/legacy-modes/mode/yaml'
import { StreamLanguage, LanguageSupport } from '@codemirror/language'
import { indentWithTab } from '@codemirror/commands'
import { oneDark } from '@codemirror/theme-one-dark'

import { Report, createDiagnostics } from '@/codemirror/diagnostics'
import { createIdler } from '@/codemirror/idle'

export type CodeMirrorOptions = {
  parent: HTMLDivElement
  initialCode: string
  setCode: (code: string) => void
}

export function CodeMirror(options: CodeMirrorOptions) {
  // const { rimu, parent, initialCode, setCode, setOutput } = options
  const { parent, initialCode, setCode } = options

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

  const diagnostics = createDiagnostics()

  const startState = EditorState.create({
    doc: initialCode,
    extensions: [basicSetup, keymap.of([indentWithTab]), oneDark, yaml, idler, diagnostics],
  })

  const view = new EditorView({ state: startState, parent })

  return view
}
