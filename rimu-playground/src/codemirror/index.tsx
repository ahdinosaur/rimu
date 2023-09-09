import { EditorState } from '@codemirror/state'
import { basicSetup } from 'codemirror'
import { EditorView, keymap } from '@codemirror/view'
import * as yamlMode from '@codemirror/legacy-modes/mode/yaml'
import { StreamLanguage, LanguageSupport } from '@codemirror/language'
import { indentWithTab } from '@codemirror/commands'
import { oneDark } from '@codemirror/theme-one-dark'

import { Report, evaler as createEvaler } from '@/codemirror/eval'

const sourceId = 'playground'

// message passing
//
// out:
// - code output
//
// in:
// - error reports
//
// how?
//
// - use commands to send messages in
//   - to send messages to codemirror
// - use "set*" functions to receive messages out
//
// so ...
//
// [ ] a setCode function passed to codemirror
// [ ] a setReports command dispatched to codemirror on change
//

export type CodeMirrorOptions = {
  parent: HTMLDivElement
  initialCode: string
  setCode: (code: string) => void
}

export function CodeMirror(options: CodeMirrorOptions) {
  // const { rimu, parent, initialCode, setCode, setOutput } = options
  const { parent, initialCode, setCode } = options

  const yaml = new LanguageSupport(StreamLanguage.define(yamlMode.yaml))

  const evaler = createEvaler(
    (view) => {
      const code = view.state.sliceDoc()
      setCode(code)

      /*

      const reports: Array<Report> = []

      let output
      try {
        output = rimu.render(code, sourceId)
      } catch (err) {
        // @ts-ignore
        if (err.reports == null) throw err

        // @ts-ignore
        for (const report of err.reports) {
          const { span } = report
          let message = report.message
          for (const [_span, label] of report.labels) {
            message += '\n\n' + label
          }
          reports.push({
            from: span.start,
            to: span.end,
            sourceId: span.sourceId,
            severity: 'error',
            message,
          })
        }
      }

      if (output !== undefined) {
        setCode(code)
        setOutput(output)
      }

      return reports
        */
    },
    {
      delay: 50,
    },
  )

  const startState = EditorState.create({
    doc: initialCode,
    extensions: [basicSetup, keymap.of([indentWithTab]), oneDark, yaml, evaler],
  })

  const view = new EditorView({ state: startState, parent })

  return view
}
