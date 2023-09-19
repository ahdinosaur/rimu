import { parser } from '../dist/index.js'
import { fileTests } from '@lezer/generator/dist/test'

import * as fs from 'fs'
import * as path from 'path'
import { fileURLToPath } from 'url'

const caseDir = path.dirname(fileURLToPath(import.meta.url))

for (const file of fs.readdirSync(caseDir)) {
  if (!/\.txt$/.test(file)) continue

  let name = /^[^\.]*/.exec(file)[0]

  describe(name, () => {
    const caseFile = fs.readFileSync(path.join(caseDir, file), 'utf8')
    for (const { name, run } of fileTests(caseFile, file)) {
      if (name !== 'List') continue
      it(name, () => run(parser))
    }
  })
}
