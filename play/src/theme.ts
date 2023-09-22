import { extendBaseTheme, ThemeConfig } from '@chakra-ui/react'
import { theme as baseTheme } from '@chakra-ui/theme'
import { variants, labels, Labels } from '@catppuccin/palette'

const { Button, List, Heading, Link, Spinner, Code, Kbd, Modal, Select, Menu, Popover, Tabs } =
  baseTheme.components

const config: ThemeConfig = {
  initialColorMode: 'system',
  useSystemColorMode: true,
}

type ColorName = keyof typeof labels
type ColorValue = {
  _light: string
  _dark: string
}
type Colors = Record<ColorName, ColorValue>
const ctp = Object.entries(labels).reduce((sofar, [name, palette]) => {
  const color = {
    _light: palette.latte.hex,
    _dark: palette.macchiato.hex,
  }
  // @ts-ignore
  sofar[name] = color
  return sofar
}, {}) as Colors

const semanticTokens = {
  colors: {
    // https://github.com/chakra-ui/chakra-ui/blob/eb0316d/packages/components/theme/src/semantic-tokens.ts
    'chakra-body-text': ctp['text'],
    'chakra-body-bg': ctp['base'],
    'chakra-border-color': ctp['surface0'],
    'chakra-inverse-text': ctp['base'],
    'chakra-subtle-bg': ctp['surface0'],
    'chakra-subtle-text': ctp['overlay1'],
    'chakra-placeholder-color': ctp['subtext1'],

    ctp,
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
    Menu,
    Popover,
    Tabs,
  },
})
