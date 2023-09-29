// @ts-ignore
import helloWorldCode from '../../examples/hello-world.rimu'
// @ts-ignore
import ifCode from '../../examples/if.rimu'
// @ts-ignore
import letCode from '../../examples/let.rimu'
// @ts-ignore
import mapCode from '../../examples/map.rimu'

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
    name: 'If',
    code: ifCode,
  },
  {
    name: 'Let',
    code: letCode,
  },
  {
    name: 'Map',
    code: mapCode,
  },
]
