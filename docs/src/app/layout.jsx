import { Footer, Layout, Navbar } from 'nextra-theme-docs'
import { Banner, Head } from 'nextra/components'
import { getPageMap } from 'nextra/page-map'
import { Logo } from '../components/logo.jsx'
import 'nextra-theme-docs/style.css'
import '../../styles.css'

export const metadata = {
  metadataBase: new URL('https://rimu.dev'),
  title: {
    default: 'Rimu',
    template: 'Rimu – %s',
  },
  description:
    'Rimu is a friendly template language for structured data and functional expressions.',
  applicationName: 'Rimu',
  appleWebApp: { title: 'Rimu' },
  other: { 'msapplication-TileColor': '#dffdff' },
  icons: {
    icon: [
      { url: '/favicon-32x32.png', sizes: '32x32', type: 'image/png' },
      { url: '/favicon-16x16.png', sizes: '16x16', type: 'image/png' },
    ],
    apple: [{ url: '/apple-touch-icon.png', sizes: '180x180' }],
    other: [{ rel: 'mask-icon', url: '/safari-pinned-tab.svg', color: '#dffdff' }],
  },
  manifest: '/site.webmanifest',
  openGraph: {
    title: 'Rimu',
    description:
      'Rimu is a friendly template language for structured data and functional expressions. 🌱',
  },
}

export const viewport = {
  themeColor: '#dffdff',
}

const banner = (
  <Banner storageKey="star-on-github">
    <a href="https://github.com/ahdinosaur/rimu" target="_blank" rel="noreferrer">
      Like this project? Star on Github! ⭐
    </a>
  </Banner>
)

const navbar = <Navbar logo={<Logo />} projectLink="https://github.com/ahdinosaur/rimu" />

const footer = <Footer>Rimu 🌱</Footer>

export default async function RootLayout({ children }) {
  return (
    <html lang="en" dir="ltr" suppressHydrationWarning>
      <Head />
      <body>
        <Layout
          banner={banner}
          navbar={navbar}
          pageMap={await getPageMap()}
          docsRepositoryBase="https://github.com/ahdinosaur/rimu/tree/main/docs"
          footer={footer}
        >
          {children}
        </Layout>
      </body>
    </html>
  )
}
