'use client'

import { useCallback, ChangeEventHandler } from 'react'
import { Code, Flex, Select } from '@chakra-ui/react'

export type Format = 'json' | 'yaml' | 'toml'

export type OutputData = any

export type OutputProps = {
  height: string
  output: OutputData
  format: Format
  setFormat: (format: Format) => void
}

export function Output(props: OutputProps) {
  const { height, output, format, setFormat } = props

  return (
    <Flex sx={{ height, flexDirection: 'column', width: '100%' }}>
      <FormatSelect format={format} setFormat={setFormat} />
      <Code sx={{ width: '100%', flexGrow: 1, backgroundColor: 'rimu.output.background' }}>
        {output}
      </Code>
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
