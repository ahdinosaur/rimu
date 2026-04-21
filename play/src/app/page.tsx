import { Suspense } from 'react'

import { Playground } from '@/components/playground'

export default function Home() {
  return (
    <main>
      <Suspense>
        <Playground />
      </Suspense>
    </main>
  )
}
