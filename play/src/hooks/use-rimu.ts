import { Format } from '@/components/output'
import { Report } from '@/codemirror/diagnostics'
import { useEffect } from 'react'

import { useLoader } from './use-loader'

const sourceId = 'playground'

export type UseRimuOptions = {
  code: string
  format: Format
  setOutput: (output: string) => void
  setReports: (reports: Array<Report>) => void
}

export function useRimu(options: UseRimuOptions) {
  const { code, format, setOutput, setReports } = options

  const rimu = useLoader(() => import('@/wasm'))

  useEffect(() => {
    if (rimu === null) return
    rimu.init()
  }, [rimu])

  useEffect(() => {
    if (rimu === null) return

    let reports: Array<Report> = []
    let output
    try {
      output = rimu.render(code, sourceId, format)
    } catch (err) {
      // @ts-ignore
      if (err.reports == null) throw err

      // @ts-ignore
      for (const report of err.reports) {
        const { span, message, labels, notes } = report

        reports.push({
          span: toCodemirrorSpan(span),
          message,
          labels: labels.map((label: any) => ({
            ...label,
            span: toCodemirrorSpan(span),
          })),
          notes,
          severity: 'error',
        })
      }
    }

    if (output !== undefined) {
      setOutput(output)
    }

    setReports(reports)
  }, [rimu, code, format, setOutput, setReports])
}

function toCodemirrorSpan(span: any) {
  const { sourceId, start, end } = span
  return { sourceId, from: start, to: end }
}
