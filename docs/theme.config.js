import { useRouter } from 'next/router'
import { Pre as NextraPre } from 'nextra/components'
import { useEffect, useRef, useState } from 'react'

export default {
  logo: (
    <div class="logo">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 72 72" className="logo__svg">
        <g
          style={{
            display: 'inline',
          }}
        >
          <ellipse
            cx={36}
            cy={36}
            rx={35.542}
            ry={35.695}
            style={{
              fill: '#bce5eb',
              fillOpacity: 1,
              stroke: '#000',
              strokeWidth: 0.75599998,
              strokeLinecap: 'round',
              strokeLinejoin: 'round',
              strokeMiterlimit: 4,
              strokeDasharray: 'none',
              strokeOpacity: 1,
            }}
          />
        </g>
        <path
          fill="#B1CC33"
          d="M51.935 35.872c4.2.928 6.765 5.482 6.765 5.482s-4.245 3.05-8.445 2.123-6.765-5.483-6.765-5.483 4.247-3.048 8.445-2.122z"
        />
        <path
          fill="#5C9E31"
          d="M22.362 19.992c4.067 1.4 6.098 6.216 6.098 6.216s-4.564 2.548-8.632 1.149-6.098-6.216-6.098-6.216 4.566-2.546 8.632-1.149z"
        />
        <path
          fill="none"
          stroke="#000"
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M36 63.166v-12l-5-22m5 22 5-10M22.362 19.992c4.067 1.4 6.098 6.216 6.098 6.216s-4.564 2.548-8.632 1.149-6.098-6.216-6.098-6.216 4.566-2.546 8.632-1.149zm29.573 15.88c4.2.928 6.765 5.482 6.765 5.482s-4.245 3.05-8.445 2.123-6.765-5.483-6.765-5.483 4.247-3.048 8.445-2.122z"
        />
      </svg>
      <span className="logo__name">Rimu</span>
    </div>
  ),
  docsRepositoryBase: 'https://github.com/ahdinosaur/rimu/tree/main/docs',
  project: {
    link: 'https://github.com/ahdinosaur/rimu',
  },
  useNextSeoProps() {
    const { asPath } = useRouter()
    if (asPath !== '/') {
      return {
        titleTemplate: 'Rimu – %s',
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
      <meta
        property="og:description"
        content="Rimu is a friendly template language for structured data and functional expressions. 🌱"
      />
    </>
  ),
  banner: {
    key: 'star-on-github',
    text: (
      <a href="https://github.com/ahdinosaur/rimu" target="_blank">
        Like this project? Star on Github! ⭐
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
    const codeEl = ref.current?.previousSibling?.querySelector('code')
    if (codeEl == null) return
    if (!codeEl.classList.contains('language-rimu')) return
    const code = codeEl.textContent
    if (code == null) return
    const url = `https://play.rimu.dev?i=u${encodeURIComponent(code)}`
    setPlaygroundUrl(url)
  }, [])

  return (
    <>
      <NextraPre {...props} />
      <aside ref={ref} className="open-play open-play__container">
        {playgroundUrl && (
          <>
            (
            <a className="open-play__link" href={playgroundUrl} target="_blank" rel="noreferrer">
              Open in Playground
            </a>
            )
          </>
        )}
      </aside>
    </>
  )
}
