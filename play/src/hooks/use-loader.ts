import { useEffect, useState } from 'react'

export function useLoader<ModuleType>(loader: () => Promise<ModuleType>): ModuleType | null {
  const [loaded, setLoaded] = useState<ModuleType | null>(null)

  useEffect(() => {
    ;(async () => {
      const loaded = await loader()
      setLoaded(loaded)
    })()
  }, [loader])

  return loaded
}
