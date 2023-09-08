'use client'

import './editor.css'

import React, { useRef, useEffect } from 'react'
import { clsx } from 'clsx'

import { EditorState } from '@codemirror/state'
import { EditorView, basicSetup } from 'codemirror'
import * as yamlMode from '@codemirror/legacy-modes/mode/yaml'
import { StreamLanguage, LanguageSupport } from '@codemirror/language'
import { oneDark } from '@codemirror/theme-one-dark'

import { Report, evaler as createEvaler } from '@/codemirror/eval'

const sourceId = 'playground'

export type EditorProps = {
  className: string
  rimu: typeof import('rimu-wasm')
  initialCode: string
  setCode: (code: string) => void
  setOutput: (output: any) => void
}

export function Editor(props: EditorProps) {
  const { className, rimu, initialCode, setCode, setOutput } = props

  const editorRef = useRef(null)

  useEffect(() => rimu.init(), [rimu])

  useEffect(() => {
    if (editorRef.current == null) return

    const yaml = new LanguageSupport(StreamLanguage.define(yamlMode.yaml))

    const evaler = createEvaler(
      (view) => {
        const reports: Array<Report> = []

        const code = view.state.sliceDoc()

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
      },
      {
        delay: 50,
      },
    )

    const startState = EditorState.create({
      doc: initialCode,
      extensions: [basicSetup, oneDark, yaml, evaler],
    })

    const view = new EditorView({ state: startState, parent: editorRef.current })

    return () => {
      view.destroy()
    }
  }, [editorRef, rimu, initialCode, setCode, setOutput])

  return <div className={className} ref={editorRef}></div>
}
