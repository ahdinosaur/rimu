import { extendBaseTheme, ThemeConfig } from '@chakra-ui/react'
import { theme as baseTheme } from '@chakra-ui/theme'

const { Button, List, Heading, Link, Spinner, Code, Kbd, Modal, Select } = baseTheme.components

const config: ThemeConfig = {
  initialColorMode: 'system',
  useSystemColorMode: true,
}

const semanticTokens = {
  colors: {
    // https://github.com/chakra-ui/chakra-ui/blob/eb0316d/packages/components/theme/src/semantic-tokens.ts
    'chakra-body-text': { _light: 'gray.800', _dark: 'whiteAlpha.900' },
    'chakra-body-bg': { _light: 'white', _dark: 'gray.800' },
    'chakra-border-color': { _light: 'gray.200', _dark: 'whiteAlpha.300' },
    'chakra-inverse-text': { _light: 'white', _dark: 'gray.800' },
    'chakra-subtle-bg': { _light: 'gray.100', _dark: 'gray.700' },
    'chakra-subtle-text': { _light: 'gray.600', _dark: 'gray.400' },
    'chakra-placeholder-color': { _light: 'gray.500', _dark: 'whiteAlpha.400' },

    'rimu-header-bg': { _light: 'gray.300', _dark: 'gray.600' },
    'rimu-output-bg': { _light: 'gray.200', _dark: 'gray.700' },

    rimu: {
      format: {
        background: { _light: 'teal.50', _dark: 'teal.800' },
        text: { _light: 'teal.800', _dark: 'teal.50' },
        border: { _light: 'teal.500', _dark: 'teal.500' },
      },
    },
  },
}

export const theme = extendBaseTheme({
  config,
  semanticTokens,
  components: {
    Button,
    List,
    Heading,
    Link,
    Spinner,
    Code,
    Kbd,
    Modal,
    Select,
  },
})
