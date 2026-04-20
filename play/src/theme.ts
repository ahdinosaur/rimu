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
          DEFAULT: { value: { _light: ctp.base.value._light, _dark: ctp.base.value._dark } },
          subtle: {
            value: { _light: ctp.mantle.value._light, _dark: ctp.mantle.value._dark },
          },
          muted: {
            value: { _light: ctp.surface0.value._light, _dark: ctp.surface0.value._dark },
          },
        },
        fg: {
          DEFAULT: { value: { _light: ctp.text.value._light, _dark: ctp.text.value._dark } },
          muted: {
            value: { _light: ctp.subtext1.value._light, _dark: ctp.subtext1.value._dark },
          },
        },
        border: {
          value: { _light: ctp.surface0.value._light, _dark: ctp.surface0.value._dark },
        },
      },
    },
  },
})

export const system = createSystem(defaultConfig, config)
