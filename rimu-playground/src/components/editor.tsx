import React, { useRef, useEffect } from 'react'
import { Box, useColorModeValue } from '@chakra-ui/react'
import { EditorView } from 'codemirror'
import { Variant } from 'codemirror-theme-catppuccin'

import { CodeMirror } from '@/codemirror'
import { Report, setReports } from '@/codemirror/diagnostics'

export type EditorProps = {
  height: string
  code: string
  setCode: (code: string) => void
  reports: Array<Report>
}

export function Editor(props: EditorProps) {
  const { height, code, setCode, reports } = props

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
      view.destroy()
    }
  }, [editorParentRef, theme, setCode])

  useEffect(() => {
    if (editorViewRef.current == null) return
    const view = editorViewRef.current

    view.dispatch(setReports(view.state, reports))
  }, [editorViewRef, reports])

  return (
    <Box
      sx={{
        width: '100%',
        height,

        '.cm-editor': {
          height: '100%',
        },

        '.cm-scroller': {
          height: '100%',
          overflowY: 'auto',
        },
      }}
      ref={editorParentRef}
    />
  )
}
