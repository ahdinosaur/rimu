import { extendBaseTheme, ThemeConfig } from '@chakra-ui/react'
import { theme as baseTheme } from '@chakra-ui/theme'
import { flavors, type ColorName } from '@catppuccin/palette'

const { Button, List, Heading, Link, Spinner, Code, Kbd, Modal, Select, Menu, Popover, Tabs } =
  baseTheme.components

const config: ThemeConfig = {
  initialColorMode: 'system',
  useSystemColorMode: true,
}

type ColorValue = {
  _light: string
  _dark: string
}
type Colors = Record<ColorName, ColorValue>
const ctp = flavors.latte.colorEntries.reduce((sofar, [name, color]) => {
  sofar[name] = {
    _light: color.hex,
    _dark: flavors.macchiato.colors[name].hex,
  }
  return sofar
}, {} as Colors)

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
