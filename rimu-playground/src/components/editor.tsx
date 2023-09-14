import React, { useRef, useEffect } from 'react'
import { Box, useColorModeValue } from '@chakra-ui/react'
import { EditorView } from 'codemirror'
import { Variant } from 'codemirror-theme-catppuccin'

import { CodeMirror, CodeMirrorState } from '@/codemirror'
import { Report, setReports } from '@/codemirror/diagnostics'

export type EditorProps = {
  height: string
  code: string
  setCode: (code: string) => void
  codeToLoad: string | null
  resetCodeToLoad: () => void
  reports: Array<Report>
}

export function Editor(props: EditorProps) {
  const { height, code, setCode, codeToLoad, resetCodeToLoad, reports } = props

  const editorParentRef = useRef(null)
  const editorViewRef = useRef<EditorView | null>(null)
  const theme = useColorModeValue<Variant>('latte', 'mocha') as Variant

  // on init
  useEffect(() => {
    if (editorParentRef.current == null) return

    const view = CodeMirror({
      parent: editorParentRef.current,
      theme,
      code,
      setCode,
    })

    editorViewRef.current = view

    return () => {
      view.destroy()
    }
  }, [editorParentRef])

  // on reports change
  useEffect(() => {
    if (editorViewRef.current == null) return
    const view = editorViewRef.current

    view.dispatch(setReports(view.state, reports))
  }, [editorViewRef, reports])

  // on theme change
  useEffect(() => {
    if (editorViewRef.current == null) return
    const view = editorViewRef.current

    const state = CodeMirrorState({
      code,
      theme,
      setCode,
    })

    view.setState(state)
  }, [editorViewRef, theme])

  // on code to load
  useEffect(() => {
    if (editorViewRef.current == null) return
    const view = editorViewRef.current
    if (codeToLoad == null) return

    const state = CodeMirrorState({
      code: codeToLoad,
      theme,
      setCode,
    })

    view.setState(state)
    resetCodeToLoad()
  }, [editorViewRef, codeToLoad])

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
