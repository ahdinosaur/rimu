'use client'

import { useCallback, useState } from 'react'
import { EditorState } from '@codemirror/state'
import {
  Box,
  Flex,
  Tab,
  TabIndicator,
  TabIndicatorProps,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
  TabsProps,
  useBreakpointValue,
} from '@chakra-ui/react'
// @ts-ignore
import { useResplit } from 'react-resplit'

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
  const bodyHeight = `calc(100dvh - ${headerHeight})`

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
    <Flex sx={{ flexDirection: 'column', height: '100dvh', alignItems: 'stretch' }}>
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

  const { getContainerProps, getSplitterProps, getPaneProps } = useResplit({
    direction: 'horizontal',
  })

  return (
    <Box {...getContainerProps()} sx={{ flexGrow: 1 }}>
      <Box {...getPaneProps(0, { initialSize: '0.5fr' })}>
        <PanelTabs
          tabProps={{
            colorScheme: 'gray',
            variant: 'unstyled',
          }}
          tabs={[
            {
              label: 'Template',
              element: editorElement,
            },
          ]}
        />
      </Box>
      <Box {...getSplitterProps(1, { size: '12px' })} sx={{ backgroundColor: 'ctp.surface0' }} />
      <Box {...getPaneProps(2, { initialSize: '0.5fr' })}>
        <PanelTabs
          tabProps={{
            colorScheme: 'gray',
            variant: 'unstyled',
          }}
          tabs={[
            {
              label: 'Output',
              element: outputElement,
            },
          ]}
        />
      </Box>
    </Box>
  )
}

function PlaygroundPanesMobile(props: PlaygroundPanesProps) {
  const { editorElement, outputElement } = props

  return (
    <PanelTabs
      tabProps={{
        colorScheme: 'green',
      }}
      tabs={[
        {
          label: 'Template',
          element: editorElement,
        },
        {
          label: 'Output',
          element: outputElement,
        },
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
  tabProps?: Omit<TabsProps, 'children'>
  tabIndicatorProps?: TabIndicatorProps
}

function PanelTabs(props: PanelTabsProps) {
  const { tabs, tabProps = {}, tabIndicatorProps } = props

  return (
    <Tabs variant="enclosed" isFitted size="sm" {...tabProps}>
      <TabList>
        {tabs.map((t, i) => (
          <Tab key={i}>{t.label}</Tab>
        ))}
      </TabList>

      {tabIndicatorProps && <TabIndicator {...tabIndicatorProps} />}

      <TabPanels>
        {tabs.map((t, i) => (
          <TabPanel key={i} sx={{ padding: 0 }}>
            {t.element}
          </TabPanel>
        ))}
      </TabPanels>
    </Tabs>
  )
}
