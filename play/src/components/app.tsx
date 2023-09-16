'use client'

import { ChakraBaseProvider, ColorModeScript } from '@chakra-ui/react'

import { theme } from '@/theme'

export type AppProps = {
  children: React.ReactNode
}

export function App(props: AppProps) {
  const { children } = props

  return (
    <ChakraBaseProvider theme={theme}>
      <ColorModeScript initialColorMode={theme.config.initialColorMode} />

      {children}
    </ChakraBaseProvider>
  )
}
