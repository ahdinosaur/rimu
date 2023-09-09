'use client'

import { CodeMirror } from '@/codemirror'

import React, { useRef, useEffect } from 'react'

import './editor.css'
import styles from './editor.module.css'

export type EditorProps = {
  rimu: typeof import('@/wasm')
  initialCode: string
  setCode: (code: string) => void
  setOutput: (output: any) => void
}

export function Editor(props: EditorProps) {
  const { rimu, initialCode, setCode, setOutput } = props

  const editorRef = useRef(null)

  useEffect(() => rimu.init(), [rimu])

  useEffect(() => {
    if (editorRef.current == null) return

    const view = CodeMirror({
      rimu,
      parent: editorRef.current,
      initialCode,
      setCode,
      setOutput,
    })

    return () => {
      view.destroy()
    }
  }, [editorRef, rimu, initialCode, setCode, setOutput])

  return <div className={styles.container} ref={editorRef}></div>
}
