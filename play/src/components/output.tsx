'use client'

import { useCallback, ChangeEventHandler } from 'react'
import { Code, Flex, Text, Select, Box, HStack } from '@chakra-ui/react'

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
      <Code sx={{ width: '100%', flexGrow: 1, backgroundColor: 'ctp.mantle' }}>
        <pre>{output}</pre>
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
    <HStack
      spacing={4}
      sx={{
        flexDirection: 'row',
        justifyContent: 'center',
        alignItems: 'baseline',
        backgroundColor: 'ctp.mantle',
        padding: 1,
      }}
    >
      <Box>
        <Select
          size="xs"
          variant="outline"
          value={format}
          onChange={handleChange}
          aria-label="Format"
          sx={{
            flexGrow: 1,
            color: 'ctp.base',
            borderColor: 'ctp.text',
            backgroundColor: 'ctp.lavender',
          }}
        >
          <option value="json">JSON</option>
          <option value="yaml">YAML</option>
          <option value="toml">TOML</option>
        </Select>
      </Box>
    </HStack>
  )
}
