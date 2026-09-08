import { readFileSync } from 'node:fs'
import process from 'node:process'
import dotenv from 'dotenv'
import dotenvExpand from 'dotenv-expand'

const [mode, path, rawIterations = '1000'] = process.argv.slice(2)
const iterations = Number(rawIterations)
let checksum = 0

if (mode === 'parse') {
  const input = readFileSync(path)
  for (let i = 0; i < iterations; i++) checksum = (checksum + Object.keys(dotenv.parse(input)).length) >>> 0
} else if (mode === 'parse-fast') {
  const input = readFileSync(path)
  for (let i = 0; i < iterations; i++) checksum = (checksum + Object.keys(dotenv.parse(input, { fast: true })).length) >>> 0
} else if (mode === 'expand') {
  const input = readFileSync(path)
  for (let i = 0; i < iterations; i++) {
    const result = dotenvExpand.expand({ parsed: dotenv.parse(input), processEnv: {} })
    checksum = (checksum + Object.keys(result.parsed).length) >>> 0
  }
} else {
  console.error('usage: node node-runner.mjs <parse|parse-fast|expand> <file> [iterations]')
  process.exit(1)
}
console.log(checksum)
