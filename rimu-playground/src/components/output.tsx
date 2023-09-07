'use client'

type OutputData = any

export type OutputProps = {
  classNames: Partial<{
    container: string
  }>
  output: OutputData
}

export function Output(props: OutputProps) {
  const { classNames = {}, output } = props

  const json = JSON.stringify(output, null, 2)

  return (
    <div className={classNames.container}>
      <code>
        <pre>{json}</pre>
      </code>
    </div>
  )
}
