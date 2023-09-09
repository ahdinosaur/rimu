'use client'

import { CodeMirror } from '@/codemirror'

import React, { useRef, useEffect } from 'react'

import './editor.css'
import styles from './editor.module.css'
import { Format } from './output'
import { EditorView } from 'codemirror'
import { Report, setReports } from '@/codemirror/eval'

const sourceId = 'playground'

export type EditorProps = {
  rimu: typeof import('@/wasm')
  initialCode: string
  format: Format
  code: string
  setCode: (code: string) => void
  setOutput: (output: any) => void
}

export function Editor(props: EditorProps) {
  const { rimu, initialCode, format, code, setCode, setOutput } = props

  const editorParentRef = useRef(null)
  const editorViewRef = useRef<EditorView | null>(null)

  useEffect(() => rimu.init(), [rimu])

  useEffect(() => {
    if (editorParentRef.current == null) return

    const view = CodeMirror({
      parent: editorParentRef.current,
      initialCode,
      setCode,
    })

    editorViewRef.current = view

    return () => {
      view.destroy()
    }
  }, [editorParentRef, rimu, initialCode, format, setCode, setOutput])

  useEffect(() => {
    if (editorViewRef.current == null) return
    const view = editorViewRef.current

    const reports: Array<Report> = []

    let output
    try {
      output = rimu.render(code, sourceId, format)
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

      view.dispatch(setReports(view.state, reports))
    }

    if (output !== undefined) {
      setOutput(output)
    }
  }, [rimu, code, format, setOutput])

  return <div className={styles.container} ref={editorParentRef}></div>
}
