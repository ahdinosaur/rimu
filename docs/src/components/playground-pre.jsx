'use client'

import { Pre as NextraPre } from 'nextra/components'
import { useEffect, useRef, useState } from 'react'

export function PlaygroundPre(props) {
  const preRef = useRef(null)
  const [playgroundUrl, setPlaygroundUrl] = useState(null)

  useEffect(() => {
    const codeEl = preRef.current?.querySelector('code')
    if (codeEl == null) return
    if (!codeEl.classList.contains('language-rimu')) return
    const code = codeEl.textContent
    if (code == null) return
    const url = `https://play.rimu.dev?i=u${encodeURIComponent(code)}`
    setPlaygroundUrl(url)
  }, [])

  return (
    <>
      <NextraPre {...props} ref={preRef} />
      {playgroundUrl && (
        <aside className="open-play open-play__container">
          (
          <a className="open-play__link" href={playgroundUrl} target="_blank" rel="noreferrer">
            Open in Playground
          </a>
          )
        </aside>
      )}
    </>
  )
}
