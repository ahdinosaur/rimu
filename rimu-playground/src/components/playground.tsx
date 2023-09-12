'use client'

import { useCallback, useEffect, useState } from 'react'
import { Box, Flex } from '@chakra-ui/react'
import { useResplit } from 'react-resplit'

import { Editor } from './editor'
import { Output, Format } from './output'
import { HeaderMenu } from './header-menu'

import { useRimu } from '@/hooks/use-rimu'
import { Report } from '@/codemirror/diagnostics'

export function Playground() {
  const initialCode = 'hello: "world"'
  const [code, setCode] = useState<string>(initialCode)
  const [codeToLoad, setCodeToLoad] = useState<string | null>(null)
  const resetCodeToLoad = useCallback(() => setCodeToLoad(null), [])
  const [output, setOutput] = useState<string>('')
  const [format, setFormat] = useState<Format>('json')
  const [reports, setReports] = useState<Array<Report>>([])

  const { getContainerProps, getSplitterProps, getPaneProps } = useResplit({
    direction: 'horizontal',
  })

  useRimu({
    code,
    format,
    setOutput,
    setReports,
  })

  const headerHeight = '2.5rem'
  const bodyHeight = 'calc(100dvh - 2.5rem)'

  return (
    <Flex sx={{ flexDirection: 'column', height: '100dvh', alignItems: 'stretch' }}>
      <HeaderMenu height={headerHeight} setCodeToLoad={setCodeToLoad} />

      <Box {...getContainerProps()} sx={{ flexGrow: 1 }}>
        <Box {...getPaneProps(0, { initialSize: '0.5fr' })}>
          <Editor
            height={bodyHeight}
            code={code}
            setCode={setCode}
            codeToLoad={codeToLoad}
            resetCodeToLoad={resetCodeToLoad}
            reports={reports}
          />
        </Box>
        <Box
          {...getSplitterProps(1, { size: '12px' })}
          sx={{ backgroundColor: 'rimu.splitter.background' }}
        />
        <Box {...getPaneProps(2, { initialSize: '0.5fr' })}>
          <Output height={bodyHeight} output={output} format={format} setFormat={setFormat} />
        </Box>
      </Box>
    </Flex>
  )
}
