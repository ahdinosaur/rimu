'use client'

import { useCallback, useMemo, useState } from 'react'

import styles from './output.module.css'

export type Format = 'json' | 'yaml' | 'toml'

export type OutputData = any

export type OutputProps = {
  output: OutputData
  format: Format
  setFormat: (format: Format) => void
}

export function Output(props: OutputProps) {
  const { output, format, setFormat } = props

  const setFormatToJson = useCallback(() => setFormat('json'), [setFormat])
  const setFormatToYaml = useCallback(() => setFormat('yaml'), [setFormat])
  const setFormatToToml = useCallback(() => setFormat('toml'), [setFormat])

  return (
    <div className={styles.container}>
      <ul className={styles.formatList}>
        <li className={styles.formatItem}>
          <button
            role="tab"
            className={styles.formatButton}
            onClick={setFormatToJson}
            aria-selected={format === 'json'}
            id="format-json"
          >
            JSON
          </button>
        </li>
        <li className={styles.formatItem}>
          <button
            role="tab"
            className={styles.formatButton}
            onClick={setFormatToYaml}
            aria-selected={format === 'yaml'}
            id="format-yaml"
          >
            YAML
          </button>
        </li>
        <li className={styles.formatItem}>
          <button
            role="tab"
            className={styles.formatButton}
            onClick={setFormatToToml}
            aria-selected={format === 'toml'}
            id="format-toml"
          >
            TOML
          </button>
        </li>
      </ul>
      <code>
        <pre>{output}</pre>
      </code>
    </div>
  )
}
