import { useRouter } from 'next/router'
import { Pre as NextraPre } from 'nextra/components'
import { useEffect, useRef, useState } from 'react'

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
        ⭐ Like what you see? Star on Github! ⭐
      </a>
    ),
  },
  components: {
    pre: Pre,
  },
}

function Pre(props) {
  const ref = useRef(null)
  const [playgroundUrl, setPlaygroundUrl] = useState(null)

  useEffect(() => {
    const codeEl = ref.current?.querySelector('code')
    if (codeEl == null) return
    if (!codeEl.classList.contains('language-rimu')) return
    const code = codeEl.textContent
    if (code == null) return
    const url = `https://play.rimu.dev?i=u${encodeURIComponent(code)}`
    setPlaygroundUrl(url)
  }, [])

  return (
    <div ref={ref}>
      <NextraPre {...props} />
      {playgroundUrl && (
        <div style={{ marginTop: '-0.5rem' }}>
          (
          <a
            className="nx-underline nx-decoration-from-font [text-underline-position:from-font] nx-mb-4"
            href={playgroundUrl}
            target="_blank"
            rel="noreferrer"
          >
            Open in Playground
          </a>
          )
        </div>
      )}
    </div>
  )
}
