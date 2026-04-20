'use client'

import { useCallback, useState } from 'react'
import { EditorState } from '@codemirror/state'
import { Flex, Tabs, useBreakpointValue } from '@chakra-ui/react'
import { Group, Panel, Separator } from 'react-resizable-panels'

import { Editor } from './editor'
import { Output, Format } from './output'
import { HeaderMenu } from './header-menu'

import { useRimu } from '@/hooks/use-rimu'
import { Report } from '@/codemirror/diagnostics'
import { useQueryParams } from '@/hooks/use-query-params'

export function Playground() {
  const [code, setCode] = useState<string>('')
  const [codeToLoad, setCodeToLoad] = useState<string | null>(null)
  const resetCodeToLoad = useCallback(() => setCodeToLoad(null), [])
  const [output, setOutput] = useState<string>('')
  const [format, setFormat] = useState<Format>('json')
  const [reports, setReports] = useState<Array<Report>>([])
  const [editorState, setEditorState] = useState<EditorState | null>(null)

  useRimu({
    code,
    format,
    setOutput,
    setReports,
  })

  useQueryParams({
    code,
    setCodeToLoad,
  })

  const headerHeight = '2.5rem'
  const bodyHeight = `calc(100dvh - ${headerHeight} - var(--chakra-sizes-9))`

  const isMobile = useBreakpointValue({ base: true, md: false })

  const editorElement = (
    <Editor
      height={bodyHeight}
      code={code}
      setCode={setCode}
      codeToLoad={codeToLoad}
      resetCodeToLoad={resetCodeToLoad}
      reports={reports}
      editorState={editorState}
      setEditorState={setEditorState}
    />
  )
  const outputElement = (
    <Output height={bodyHeight} output={output} format={format} setFormat={setFormat} />
  )

  const Panels = isMobile ? PlaygroundPanesMobile : PlaygroundPanesDesktop

  return (
    <Flex flexDirection="column" height="100dvh" alignItems="stretch">
      <HeaderMenu height={headerHeight} setCodeToLoad={setCodeToLoad} />
      <Panels editorElement={editorElement} outputElement={outputElement} />
    </Flex>
  )
}

type PlaygroundPanesProps = {
  editorElement: React.ReactElement
  outputElement: React.ReactElement
}

function PlaygroundPanesDesktop(props: PlaygroundPanesProps) {
  const { editorElement, outputElement } = props

  return (
    <Group orientation="horizontal" style={{ flexGrow: 1 }}>
      <Panel defaultSize={50}>
        <PanelTabs
          tabsProps={{ colorPalette: 'gray', variant: 'plain' }}
          tabs={[{ label: 'Template', element: editorElement }]}
        />
      </Panel>
      <Separator
        style={{ width: '12px', backgroundColor: 'var(--chakra-colors-ctp-surface0)' }}
      />
      <Panel defaultSize={50}>
        <PanelTabs
          tabsProps={{ colorPalette: 'gray', variant: 'plain' }}
          tabs={[{ label: 'Output', element: outputElement }]}
        />
      </Panel>
    </Group>
  )
}

function PlaygroundPanesMobile(props: PlaygroundPanesProps) {
  const { editorElement, outputElement } = props

  return (
    <PanelTabs
      tabsProps={{ colorPalette: 'purple' }}
      tabs={[
        { label: 'Template', element: editorElement },
        { label: 'Output', element: outputElement },
      ]}
    />
  )
}

type PanelTab = {
  label: string
  element: React.ReactElement
}

type PanelTabsProps = {
  tabs: Array<PanelTab>
  tabsProps?: Omit<Tabs.RootProps, 'children'>
}

function PanelTabs(props: PanelTabsProps) {
  const { tabs, tabsProps = {} } = props
  const defaultValue = tabs[0]?.label

  return (
    <Tabs.Root variant="enclosed" fitted size="sm" defaultValue={defaultValue} {...tabsProps}>
      <Tabs.List>
        {tabs.map((t) => (
          <Tabs.Trigger key={t.label} value={t.label}>
            {t.label}
          </Tabs.Trigger>
        ))}
      </Tabs.List>

      {tabs.map((t) => (
        <Tabs.Content key={t.label} value={t.label} padding={0}>
          {t.element}
        </Tabs.Content>
      ))}
    </Tabs.Root>
  )
}
