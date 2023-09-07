import dynamic from 'next/dynamic'
import { ReactNode } from 'react'

export type RimuModule = typeof import('rimu-wasm')

export type RimuProps = {
  render: (rimu: RimuModule) => ReactNode
}

export const Rimu = dynamic({
  loader: async () => {
    const rimu = await import('rimu-wasm')

    return (props: RimuProps) => {
      return props.render(rimu)
    }
  },
})
