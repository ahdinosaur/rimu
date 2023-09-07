'use client'

import { Suspense, useState } from 'react'

import { Rimu } from './rimu'
import { Editor } from './editor'
import { Output } from './output'

export function Playground() {
  const initialCode = 'hello: "world"'

  const [code, setCode] = useState<string>(initialCode)
  const [output, setOutput] = useState<any>({ hello: 'world' })

  return (
    <div className="flex flex-row">
      <Suspense>
        <Rimu
          render={(rimu) => (
            <Editor rimu={rimu} initialCode={initialCode} setCode={setCode} setOutput={setOutput} />
          )}
        />
      </Suspense>
      <Output output={output} />
    </div>
  )
}
