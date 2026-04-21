import { createSystem, defaultConfig, defineConfig } from '@chakra-ui/react'
import { flavors, type ColorName } from '@catppuccin/palette'

type SemanticColorTokens = Record<ColorName, { value: { _light: string; _dark: string } }>

const ctp = flavors.latte.colorEntries.reduce((acc, [name, color]) => {
  acc[name] = {
    value: {
      _light: color.hex,
      _dark: flavors.macchiato.colors[name].hex,
    },
  }
  return acc
}, {} as SemanticColorTokens)

const config = defineConfig({
  theme: {
    semanticTokens: {
      colors: {
        ctp,
        bg: {
          DEFAULT: ctp.base,
          subtle: ctp.mantle,
          muted: ctp.surface0,
          emphasized: ctp.surface1,
          panel: ctp.crust,
          inverted: ctp.text,
        },
        fg: {
          DEFAULT: ctp.text,
          subtle: ctp.subtext0,
          muted: ctp.subtext1,
          inverted: ctp.base,
        },
        border: {
          DEFAULT: ctp.surface1,
          muted: ctp.surface0,
          emphasized: ctp.overlay0,
        },
      },
    },
  },
})

export const system = createSystem(defaultConfig, config)
