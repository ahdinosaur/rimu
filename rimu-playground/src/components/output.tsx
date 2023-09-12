'use client'

import { useCallback, ChangeEventHandler, useEffect, useMemo } from 'react'
import { Code, Flex, Select } from '@chakra-ui/react'

import hljs from 'highlight.js/lib/core'
import json from 'highlight.js/lib/languages/javascript'
import yaml from 'highlight.js/lib/languages/yaml'
import toml from 'highlight.js/lib/languages/ini'

import '@catppuccin/highlightjs/css/catppuccin-latte.css'
import { Variant } from 'codemirror-theme-catppuccin'

hljs.registerLanguage('json', json)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('toml', toml)

export type Format = 'json' | 'yaml' | 'toml'

export type OutputData = any

export type OutputProps = {
  height: string
  theme: Variant
  output: OutputData
  format: Format
  setFormat: (format: Format) => void
}

export function Output(props: OutputProps) {
  const { height, theme, output, format, setFormat } = props

  const highlighted = useMemo(() => {
    return hljs.highlight(output, { language: format }).value
  }, [output, format])

  return (
    <Flex sx={{ height, flexDirection: 'column', width: '100%' }}>
      <FormatSelect format={format} setFormat={setFormat} />
      <pre>
        <Code
          sx={{ width: '100%', flexGrow: 1, backgroundColor: 'rimu.output.background' }}
          dangerouslySetInnerHTML={{ __html: highlighted }}
        />
      </pre>
    </Flex>
  )
}

export type FormatSelectProps = {
  format: Format
  setFormat: (format: Format) => void
}

export function FormatSelect(props: FormatSelectProps) {
  const { format, setFormat } = props

  const handleChange = useCallback<ChangeEventHandler<HTMLSelectElement>>(
    (ev) => {
      setFormat(ev.target.value as Format)
    },
    [setFormat],
  )

  return (
    <Select
      variant="outline"
      value={format}
      onChange={handleChange}
      sx={{
        color: 'rimu.format.text',
        borderColor: 'rimu.format.border',
        backgroundColor: 'rimu.format.background',
      }}
    >
      <option value="json">JSON</option>
      <option value="yaml">YAML</option>
      <option value="toml">TOML</option>
    </Select>
  )
}
