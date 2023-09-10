import React, { useRef, useEffect } from 'react'
import { Box } from '@chakra-ui/react'
import { EditorView } from 'codemirror'

import { CodeMirror } from '@/codemirror'
import { Report, setReports } from '@/codemirror/diagnostics'

import './editor.css'

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

  return <Box sx={{ width: '50%' }} ref={editorParentRef} />
}
