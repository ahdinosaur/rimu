import './globals.css'

import type { Metadata, Viewport } from 'next'
import { App } from '@/components/app'

export const metadata: Metadata = {
  title: 'Rimu Playground',
  description: 'A playground for the Rimu data template language.',
}

export const viewport: Viewport = {
  themeColor: '#dffdff',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <App>{children}</App>
      </body>
    </html>
  )
}
