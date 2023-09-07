'use client'

import { useState } from 'react'

import { useLoader } from '@/hooks/use-loader'

import { Editor } from './editor'
import { Output } from './output'

export function Playground() {
  const initialCode = 'hello: "world"'

  const rimu = useLoader(() => import('rimu-wasm'))
  const [code, setCode] = useState<string>(initialCode)
  const [output, setOutput] = useState<any>({ hello: 'world' })

  if (rimu === null) {
    return <div>Loading</div>
  }

  return (
    <div className="flex flex-row">
      <Editor
        className="w-1/2 h-screen"
        rimu={rimu}
        initialCode={initialCode}
        setCode={setCode}
        setOutput={setOutput}
      />
      <Output
        classNames={{
          container: 'w-1/2 h-screen',
        }}
        output={output}
      />
    </div>
  )
}
