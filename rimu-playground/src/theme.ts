import { extendBaseTheme, ThemeConfig } from '@chakra-ui/react'
import { theme as baseTheme } from '@chakra-ui/theme'

const { Button, List, Heading, Link, Spinner, Code, Kbd, Modal, Badge } = baseTheme.components

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

    'rimu-header-bg': { _light: 'gray.200', _dark: 'gray.600' },
    'rimu-output-header-bg': { _light: 'gray.100', _dark: 'gray.900' },
    'rimu-output-code-bg': { _light: 'gray.50', _dark: 'gray.800' },
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
    Badge,
  },
})
