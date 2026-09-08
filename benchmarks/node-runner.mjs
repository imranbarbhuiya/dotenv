import { readFileSync } from 'node:fs'
import process from 'node:process'
import dotenv from 'dotenv'
import DotEnv from 'dotenv-extra'

const [mode, path, rawIterations = '1000'] = process.argv.slice(2)
const iterations = Number(rawIterations)
let checksum = 0

if (mode === 'parse') {
  const input = readFileSync(path)
  for (let i = 0; i < iterations; i++) checksum = (checksum + Object.keys(dotenv.parse(input)).length) >>> 0
} else if (mode === 'parse-fast') {
  const input = readFileSync(path)
  for (let i = 0; i < iterations; i++) checksum = (checksum + Object.keys(dotenv.parse(input, { fast: true })).length) >>> 0
} else if (mode === 'extra') {
  for (let i = 0; i < iterations; i++) {
    const editor = new DotEnv(path)
    editor.upsert('BENCHMARK_TARGET', 'updated value')
    const renderedLength = editor.lines.join(editor.lbChar).length + (editor.newLine ? editor.lbChar.length : 0)
    checksum = (checksum + Object.keys(editor.dump()).length + renderedLength) >>> 0
  }
} else {
  console.error('usage: node node-runner.mjs <parse|parse-fast|extra> <file> [iterations]')
  process.exit(1)
}
console.log(checksum)
