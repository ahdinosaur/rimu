// @ts-ignore
import helloWorldCode from '../../examples/hello-world.rimu'
// @ts-ignore
import ifCode from '../../examples/if.rimu'
// @ts-ignore
import letCode from '../../examples/let.rimu'

export type Example = {
  name: string
  code: string
}

export const examples: Array<Example> = [
  {
    name: 'Hello world',
    code: helloWorldCode,
  },
  {
    name: 'If block',
    code: ifCode,
  },
  {
    name: 'Let block',
    code: letCode,
  },
]
