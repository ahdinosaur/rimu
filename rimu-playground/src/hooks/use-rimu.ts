import { Format } from '@/components/output'
import { useLoader } from './use-loader'
import { Report } from '@/codemirror/eval'
import { useEffect } from 'react'

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

    let output
    try {
      output = rimu.render(code, sourceId, format)
    } catch (err) {
      // @ts-ignore
      if (err.reports == null) throw err

      const reports: Array<Report> = []
      // @ts-ignore
      for (const report of err.reports) {
        const { span } = report
        let message = report.message
        for (const [_span, label] of report.labels) {
          message += '\n\n' + label
        }
        reports.push({
          from: span.start,
          to: span.end,
          sourceId: span.sourceId,
          severity: 'error',
          message,
        })
      }
      setReports(reports)
    }

    if (output !== undefined) {
      setOutput(output)
    }
  }, [rimu, code, format, setOutput, setReports])
}
