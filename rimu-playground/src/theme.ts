import { extendBaseTheme, ThemeConfig } from '@chakra-ui/react'
import { theme as baseTheme } from '@chakra-ui/theme'
import { variants as colorVariants, variants } from '@catppuccin/palette'

const { Button, List, Heading, Link, Spinner, Code, Kbd, Modal, Select, Menu } =
  baseTheme.components

const config: ThemeConfig = {
  initialColorMode: 'system',
  useSystemColorMode: true,
}

type Variant = keyof typeof variants
type ColorName = keyof (typeof variants)[Variant]
const color = (name: ColorName) => ({
  _light: colorVariants.latte[name].hex,
  _dark: colorVariants.frappe[name].hex,
})

const semanticTokens = {
  colors: {
    // https://github.com/chakra-ui/chakra-ui/blob/eb0316d/packages/components/theme/src/semantic-tokens.ts
    'chakra-body-text': color('text'),
    'chakra-body-bg': color('base'),
    'chakra-border-color': color('surface0'),
    'chakra-inverse-text': color('base'),
    'chakra-subtle-bg': color('surface0'),
    'chakra-subtle-text': color('overlay1'),
    'chakra-placeholder-color': color('subtext1'),

    rimu: {
      header: {
        background: color('crust'),
      },
      splitter: {
        background: color('surface0'),
      },
      output: {
        background: color('mantle'),
        border: color('crust'),
      },
      format: {
        /*
        background: { _light: 'teal.50', _dark: 'teal.800' },
        text: { _light: 'teal.800', _dark: 'teal.50' },
        border: { _light: 'teal.500', _dark: 'teal.500' },
        */
        background: color('lavender'),
        text: color('base'),
        border: color('text'),
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
    Menu,
  },
})
