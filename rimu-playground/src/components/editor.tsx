'use client'

import { CodeMirror } from '@/codemirror'

import React, { useRef, useEffect } from 'react'

import './editor.css'
import styles from './editor.module.css'
import { EditorView } from 'codemirror'
import { Report, setReports } from '@/codemirror/diagnostics'

export type EditorProps = {
  initialCode: string
  setCode: (code: string) => void
  reports: Array<Report>
}

export function Editor(props: EditorProps) {
  const { initialCode, setCode, reports } = props

  const editorParentRef = useRef(null)
  const editorViewRef = useRef<EditorView | null>(null)

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
  }, [editorParentRef, initialCode, setCode])

  useEffect(() => {
    if (editorViewRef.current == null) return
    const view = editorViewRef.current

    view.dispatch(setReports(view.state, reports))
  }, [editorViewRef, reports])

  return <div className={styles.container} ref={editorParentRef}></div>
}
