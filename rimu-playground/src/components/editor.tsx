import React, { useRef, useEffect } from 'react'
import { Box, useColorModeValue } from '@chakra-ui/react'
import { EditorView } from 'codemirror'
import { Variant } from 'codemirror-theme-catppuccin'

import { CodeMirror } from '@/codemirror'
import { Report, setReports } from '@/codemirror/diagnostics'

import './editor.css'

export type EditorProps = {
  code: string
  setCode: (code: string) => void
  reports: Array<Report>
}

export function Editor(props: EditorProps) {
  const { code, setCode, reports } = props

  const editorParentRef = useRef(null)
  const editorViewRef = useRef<EditorView | null>(null)
  const theme = useColorModeValue<Variant>('latte', 'mocha') as Variant

  useEffect(() => {
    if (editorParentRef.current == null) return

    const view = CodeMirror({
      parent: editorParentRef.current,
      theme,
      initialCode: code,
      setCode,
    })

    editorViewRef.current = view

    return () => {
      console.log('dessprorryy')
      view.destroy()
    }
  }, [editorParentRef, theme, setCode])

  useEffect(() => {
    if (editorViewRef.current == null) return
    const view = editorViewRef.current

    view.dispatch(setReports(view.state, reports))
  }, [editorViewRef, reports])

  return <Box sx={{ width: '50%' }} ref={editorParentRef} />
}
