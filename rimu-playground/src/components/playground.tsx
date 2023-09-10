'use client'

import { useState } from 'react'
import { Flex } from '@chakra-ui/react'

import { Editor } from './editor'
import { Output, Format } from './output'
import { HeaderMenu } from './header-menu'

import { useRimu } from '@/hooks/use-rimu'
import { Report } from '@/codemirror/diagnostics'

export function Playground() {
  const initialCode = 'hello: "world"'

  const [code, setCode] = useState<string>(initialCode)
  const [output, setOutput] = useState<string>('')
  const [format, setFormat] = useState<Format>('json')
  const [reports, setReports] = useState<Array<Report>>([])

  useRimu({
    code,
    format,
    setOutput,
    setReports,
  })

  return (
    <Flex sx={{ flexDirection: 'column', height: '100dvh', alignItems: 'stretch' }}>
      <HeaderMenu />

      <Flex sx={{ flexDirection: 'row', flexGrow: 1 }}>
        <Editor initialCode={initialCode} setCode={setCode} reports={reports} />
        <Output output={output} format={format} setFormat={setFormat} />
      </Flex>
    </Flex>
  )
}
