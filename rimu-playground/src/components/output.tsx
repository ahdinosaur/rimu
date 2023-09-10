'use client'

import { useCallback } from 'react'
import { ListItem, Button, HStack, List, Code, Flex, Text } from '@chakra-ui/react'

export type Format = 'json' | 'yaml' | 'toml'

export type OutputData = any

export type OutputProps = {
  output: OutputData
  format: Format
  setFormat: (format: Format) => void
}

export function Output(props: OutputProps) {
  const { output, format, setFormat } = props

  return (
    <Flex sx={{ flexDirection: 'column', width: '50%', height: 'full', alignItems: 'stretch' }}>
      <HStack sx={{ backgroundColor: 'rimu-output-header-bg' }}>
        <Text>Output:</Text>
        <List as={HStack} sx={{ justifyContent: 'flex-start' }}>
          <ListItem>
            <FormatButton buttonFormat="json" outputFormat={format} setFormat={setFormat}>
              JSON
            </FormatButton>
          </ListItem>
          <ListItem>
            <FormatButton buttonFormat="yaml" outputFormat={format} setFormat={setFormat}>
              YAML
            </FormatButton>
          </ListItem>
          <ListItem>
            <FormatButton buttonFormat="toml" outputFormat={format} setFormat={setFormat}>
              TOML
            </FormatButton>
          </ListItem>
        </List>
      </HStack>
      <Code sx={{ flexGrow: 1, backgroundColor: 'rimu-output-code-bg' }}>{output}</Code>
    </Flex>
  )
}

type FormatButtonProps = {
  outputFormat: Format
  buttonFormat: Format
  setFormat: (format: Format) => void
  children: React.ReactNode
}

function FormatButton(props: FormatButtonProps) {
  const { outputFormat, buttonFormat, setFormat, children } = props

  const setButtonFormat = useCallback(() => setFormat(buttonFormat), [buttonFormat, setFormat])

  const isSelected = outputFormat === buttonFormat

  return (
    <Button
      role="tab"
      colorScheme={isSelected ? 'purple' : 'teal'}
      onClick={setButtonFormat}
      aria-selected={isSelected}
      id={`format-${buttonFormat}`}
    >
      {children}
    </Button>
  )
}
