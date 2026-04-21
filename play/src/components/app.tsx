'use client'

import { ChakraProvider } from '@chakra-ui/react'
import { ThemeProvider } from 'next-themes'

import { system } from '@/theme'

export type AppProps = {
  children: React.ReactNode
}

export function App(props: AppProps) {
  const { children } = props

  return (
    <ChakraProvider value={system}>
      <ThemeProvider attribute="class" disableTransitionOnChange>
        {children}
      </ThemeProvider>
    </ChakraProvider>
  )
}
