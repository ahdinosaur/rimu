import { useRouter } from 'next/router'

export default {
  logo: <span>Rimu</span>,
  docsRepositoryBase: 'https://github.com/ahdinosaur/rimu/tree/main/docs',
  project: {
    link: 'https://github.com/ahdinosaur/rimu',
  },
  useNextSeoProps() {
    const { asPath } = useRouter()
    if (asPath !== '/') {
      return {
        titleTemplate: '%s – Rimu',
      }
    }
  },
  head: (
    <>
      <meta name="viewport" content="width=device-width, initial-scale=1.0" />

      <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
      <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
      <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
      <link rel="manifest" href="/site.webmanifest" />
      <link rel="mask-icon" href="/safari-pinned-tab.svg" color="#dffdff" />
      <meta name="msapplication-TileColor" content="#dffdff" />
      <meta name="theme-color" content="#dffdff" />

      <meta property="og:title" content="Rimu" />
      <meta property="og:description" content="Functional templates for config data strutures 🌱" />
    </>
  ),
  banner: {
    key: 'star-on-github',
    text: (
      <a href="https://github.com/ahdinosaur/rimu" target="_blank">
        ⭐ Star Rimu on Github! ⭐
      </a>
    ),
  },
}
