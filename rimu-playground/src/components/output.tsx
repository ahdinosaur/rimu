'use client'

type OutputData = any

export type OutputProps = {
  output: OutputData
}

export function Output(props: OutputProps) {
  const { output } = props

  const json = JSON.stringify(output, null, 2)

  return (
    <code>
      <pre>{json}</pre>
    </code>
  )
}
