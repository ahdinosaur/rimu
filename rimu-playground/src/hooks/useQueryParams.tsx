import { useEffect } from 'react'
import { useDebounce } from 'use-debounce'
import {
  encode as encodeBase64WithGzipCompression,
  decode as decodeBase64WithGzipCompression,
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
      const searchParams = new URLSearchParams(location.search)
      const serializedCode = await serializeCode(
        CodeSerializationFormat.Base64WithGzipCompression,
        debouncedCode,
      )
      searchParams.set('i', serializedCode)
      history.replaceState(null, '', `?${searchParams}`)
      document.location.search
    })()
  }, [debouncedCode])
}

enum CodeSerializationFormat {
  Base64WithGzipCompression = 'b1', // base64 version 1
}

async function deserializeCode(serialized: string) {
  const formatSigil = serialized.substring(0, 2)
  const serializedCode = serialized.substring(2)

  switch (formatSigil) {
    case CodeSerializationFormat.Base64WithGzipCompression:
      return decodeBase64WithGzipCompression(serializedCode)
    default:
      throw new Error('Unexpected code serialization format.')
  }
}

async function serializeCode(formatSigil: CodeSerializationFormat, code: string) {
  switch (formatSigil) {
    // base64 version 1 (with browser `gzip` compression)
    case CodeSerializationFormat.Base64WithGzipCompression: {
      const serializedCode = await encodeBase64WithGzipCompression(code)
      return `${formatSigil}${serializedCode}`
    }
    default:
      throw new Error('Unexpected code serialization format.')
  }
}
