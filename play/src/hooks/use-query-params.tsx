import { usePathname, useRouter, useSearchParams } from 'next/navigation'
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

  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()

  useEffect(() => {
    ;(async () => {
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
      const nextSearchParams = new URLSearchParams(searchParams)
      nextSearchParams.set('i', serializedCode)
      const nextUrl = `${pathname}?${nextSearchParams}`
      router.replace(nextUrl)
    })()
  }, [debouncedCode])
}

enum CodeSerializationFormat {
  // url-safe base64, `deflate-raw` compression
  Base64WithDeflateRawCompression = 'b',
  // encoded with `encodeURIComponent`
  EncodedUriComponent = 'u',
}

async function deserializeCode(serialized: string) {
  const formatSigil = serialized.substring(0, 1)
  const serializedCode = serialized.substring(1)

  switch (formatSigil) {
    case CodeSerializationFormat.Base64WithDeflateRawCompression:
      return decodeBase64WithCompression(serializedCode, 'deflate-raw')
    case CodeSerializationFormat.EncodedUriComponent:
      return decodeURIComponent(serializedCode)
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
    case CodeSerializationFormat.EncodedUriComponent: {
      const serializedCode = encodeURIComponent(code)
      return `${formatSigil}${serializedCode}`
    }
    default:
      throw new Error('Unexpected code serialization format.')
  }
}
