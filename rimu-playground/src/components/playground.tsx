'use client'

import { useState } from 'react'

import { Editor } from './editor'
import { Output, Format } from './output'

import styles from './playground.module.css'
import { useRimu } from '@/hooks/use-rimu'
import { Report } from '@/codemirror/eval'

export function Playground() {
  const initialCode = 'hello: "world"'

  const [code, setCode] = useState<string>(initialCode)
  const [output, setOutput] = useState<string>('')
  const [format, setFormat] = useState<Format>('json')
  const [reports, setReports] = useState<Array<Report>>([])

  useRimu({
    code,
    format,
    setOutput,
    setReports,
  })

  return (
    <div className={styles.container}>
      <div className={styles.menu}>
        <h1 className={styles.heading}>Rimu</h1>
      </div>
      <div className={styles.panels}>
        <Editor initialCode={initialCode} setCode={setCode} reports={reports} />
        <Output output={output} format={format} setFormat={setFormat} />
      </div>
    </div>
  )
}
