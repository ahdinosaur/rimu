import { useEffect } from 'react'
import { useDebounce } from 'use-debounce'
import {
  encode as encodeBase64WithCompression,
  decode as decodeBase64WithCompression,
} from 'base64-compressor'

type UseQueryParamsOptions = {
  code: string
  setCodeToLoad: (code: string) => void
}

export function useQueryParams(options: UseQueryParamsOptions) {
  const { code, setCodeToLoad } = options

  useEffect(() => {
    ;(async () => {
      const searchParams = new URLSearchParams(location.search)
      const serializedCode = searchParams.get('i')
      if (serializedCode != null) {
        const code = await deserializeCode(serializedCode)
        setCodeToLoad(code)
      }
    })()
  }, [setCodeToLoad])

  const [debouncedCode] = useDebounce(code, 1000)

  useEffect(() => {
    if (debouncedCode == '') return
    ;(async () => {
      const serializedCode = await serializeCode(
        CodeSerializationFormat.Base64WithDeflateRawCompression,
        debouncedCode,
      )
      const searchParams = new URLSearchParams(location.search)
      searchParams.set('i', serializedCode)
      history.replaceState(null, '', `?${searchParams}`)
      document.location.search
    })()
  }, [debouncedCode])
}

enum CodeSerializationFormat {
  // url-safe base64, `deflate-raw` compression
  Base64WithDeflateRawCompression = 'b',
}

async function deserializeCode(serialized: string) {
  const formatSigil = serialized.substring(0, 1)
  const serializedCode = serialized.substring(1)

  switch (formatSigil) {
    case CodeSerializationFormat.Base64WithDeflateRawCompression:
      return decodeBase64WithCompression(serializedCode, 'deflate-raw')
    default:
      throw new Error('Unexpected code serialization format.')
  }
}

async function serializeCode(formatSigil: CodeSerializationFormat, code: string) {
  switch (formatSigil) {
    case CodeSerializationFormat.Base64WithDeflateRawCompression: {
      const serializedCode = await encodeBase64WithCompression(code, 'deflate-raw')
      return `${formatSigil}${serializedCode}`
    }
    default:
      throw new Error('Unexpected code serialization format.')
  }
}
