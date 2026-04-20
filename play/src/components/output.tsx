'use client'

import { useCallback, ChangeEventHandler } from 'react'
import { Code, Flex, NativeSelect, HStack } from '@chakra-ui/react'

export type Format = 'json' | 'yaml' | 'toml'

export type OutputData = string

export type OutputProps = {
  height: string
  output: OutputData
  format: Format
  setFormat: (format: Format) => void
}

export function Output(props: OutputProps) {
  const { height, output, format, setFormat } = props

  return (
    <Flex height={height} flexDirection="column" width="100%">
      <FormatSelect format={format} setFormat={setFormat} />
      <Code width="100%" flexGrow={1} bg="ctp.mantle">
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
      gap={4}
      flexDirection="row"
      justifyContent="center"
      alignItems="baseline"
      bg="ctp.mantle"
      padding={1}
    >
      <NativeSelect.Root size="xs" variant="outline" width="auto">
        <NativeSelect.Field
          value={format}
          onChange={handleChange}
          aria-label="Format"
          color="ctp.base"
          borderColor="ctp.text"
          bg="ctp.lavender"
        >
          <option value="json">JSON</option>
          <option value="yaml">YAML</option>
          <option value="toml">TOML</option>
        </NativeSelect.Field>
        <NativeSelect.Indicator />
      </NativeSelect.Root>
    </HStack>
  )
}
