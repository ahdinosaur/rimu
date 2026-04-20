import React, { useRef, useEffect, useState } from 'react'
import { Box, useColorModeValue } from '@chakra-ui/react'
import { EditorState } from '@codemirror/state'
import { EditorView } from 'codemirror'
import { Variant } from 'codemirror-theme-catppuccin'
// @ts-ignore
import { useResplit } from 'react-resplit'

import { CodeMirror, updateCode, updateTheme } from '@/codemirror'
import { Report, setReports } from '@/codemirror/diagnostics'
import { DiagnosticPanel } from './diagnostic'

export type EditorProps = {
  height: string
  code: string
  setCode: (code: string) => void
  codeToLoad: string | null
  resetCodeToLoad: () => void
  reports: Array<Report>
  editorState: EditorState | null
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
    view.dispatch(setReports(reports))
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

  const { getContainerProps, getSplitterProps, getPaneProps } = useResplit({
    direction: 'vertical',
  })

  return (
    <Box {...getContainerProps()} sx={{ height }}>
      <Box {...getPaneProps(0, { initialSize: '0.8fr' })}>
        <Box
          sx={{
            width: '100%',
            height: '100%',

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
      </Box>
      <Box {...getSplitterProps(1, { size: '12px' })} sx={{ backgroundColor: 'ctp.surface0' }} />
      <Box {...getPaneProps(2, { initialSize: '0.2fr' })}>
        <DiagnosticPanel reports={reports} />
      </Box>
    </Box>
  )
}
