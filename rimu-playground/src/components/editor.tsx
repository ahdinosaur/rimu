'use client'

import { CodeMirror } from '@/codemirror'
import './editor.css'

import React, { useRef, useEffect } from 'react'
// import { clsx } from 'clsx'

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

  return <div className={className} ref={editorRef}></div>
}
