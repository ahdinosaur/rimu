'use client'

import { useState } from 'react'

import { useLoader } from '@/hooks/use-loader'

import { Editor } from './editor'
import { Output, Format } from './output'

import styles from './playground.module.css'

export function Playground() {
  const initialCode = 'hello: "world"'

  const rimu = useLoader(() => import('@/wasm'))
  const [code, setCode] = useState<string>(initialCode)
  const [output, setOutput] = useState<any>({ hello: 'world' })
  const [format, setFormat] = useState<Format>('json')

  if (rimu === null) {
    return <div className="loading">Loading</div>
  }

  return (
    <div className={styles.container}>
      <div className={styles.menu}>
        <h1 className={styles.heading}>Rimu</h1>
      </div>
      <div className={styles.panels}>
        <Editor
          rimu={rimu}
          initialCode={initialCode}
          format={format}
          code={code}
          setCode={setCode}
          setOutput={setOutput}
        />
        <Output output={output} format={format} setFormat={setFormat} />
      </div>
    </div>
  )
}
