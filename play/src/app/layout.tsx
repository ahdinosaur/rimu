import './globals.css'

import type { Metadata } from 'next'
import { App } from '@/components/app'

export const metadata: Metadata = {
  title: 'Rimu Playground',
  description: 'A playground for the Rimu data template language.',
  themeColor: '#dffdff',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <App>{children}</App>
      </body>
    </html>
  )
}
