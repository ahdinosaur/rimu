'use client'

import { useCallback, useMemo, useState } from 'react'
import { stringify as yamlStringify } from 'yaml'
import { stringify as tomlStringify } from '@ltd/j-toml'

import styles from './output.module.css'

type OutputData = any

type Format = 'json' | 'yaml' | 'toml'

export type OutputProps = {
  output: OutputData
}

export function Output(props: OutputProps) {
  const { output } = props

  const [format, setFormat] = useState<Format>('json')
  const setFormatToJson = useCallback(() => setFormat('json'), [])
  const setFormatToYaml = useCallback(() => setFormat('yaml'), [])
  const setFormatToToml = useCallback(() => setFormat('toml'), [])

  const string = useMemo(() => {
    switch (format) {
      case 'json':
        return JSON.stringify(output, null, 2)
      case 'yaml':
        return yamlStringify(output)
      case 'toml':
        return tomlStringify(output, {
          newline: '\n',
          newlineAround: 'section',
          indent: 2,
        }).trimStart()
    }
  }, [output, format])

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
        <pre>{string}</pre>
      </code>
    </div>
  )
}
