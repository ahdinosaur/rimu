import React, { useRef, useEffect, useState } from 'react'
import { Box, useColorModeValue } from '@chakra-ui/react'
import { EditorState } from '@codemirror/state'
import { EditorView } from 'codemirror'
import { Variant } from 'codemirror-theme-catppuccin'

import { CodeMirror, updateCode, updateTheme } from '@/codemirror'
import { Report, setReports } from '@/codemirror/diagnostics'

export type EditorProps = {
  height: string
  code: string
  setCode: (code: string) => void
  codeToLoad: string | null
  resetCodeToLoad: () => void
  reports: Array<Report>
  editorState: EditorState
  setEditorState: (state: EditorState) => void
}

export function Editor(props: EditorProps) {
  const {
    height,
    code,
    setCode,
    codeToLoad,
    resetCodeToLoad,
    reports,
    editorState,
    setEditorState,
  } = props

  const parentRef = useRef(null)
  const [view, setView] = useState<EditorView | null>(null)
  const theme = useColorModeValue<Variant>('latte', 'mocha') as Variant

  // on init
  useEffect(() => {
    if (parentRef.current == null) return
    const parent = parentRef.current

    const state =
      editorState != null
        ? editorState
        : CodeMirror({
            setCode,
            theme,
            code,
            setState: setEditorState,
          })

    const view = new EditorView({ state, parent })
    setView(view)

    return () => {
      view.destroy()
      setView(null)
    }
  }, [parentRef, setCode])

  // on state change

  // on reports change
  useEffect(() => {
    if (view == null) return
    view.dispatch(setReports(view.state, reports))
  }, [view, reports])

  // on theme change
  useEffect(() => {
    if (view == null) return
    updateTheme(view, theme)
  }, [view, theme])

  // on code to load
  useEffect(() => {
    if (view == null) return
    if (codeToLoad == null) return
    updateCode(view, codeToLoad)
    resetCodeToLoad()
  }, [view, codeToLoad, resetCodeToLoad])

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
      ref={parentRef}
    />
  )
}
