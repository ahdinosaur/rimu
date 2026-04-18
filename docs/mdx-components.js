import { useMDXComponents as getDocsMDXComponents } from 'nextra-theme-docs'
import { PlaygroundPre } from './src/components/playground-pre.jsx'

const docsComponents = getDocsMDXComponents()

export function useMDXComponents(components) {
  return {
    ...docsComponents,
    ...components,
    pre: PlaygroundPre,
  }
}
