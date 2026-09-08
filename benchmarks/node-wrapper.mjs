import { readFileSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import dotenv from 'dotenv'

const [implementation, path, command, ...args] = process.argv.slice(2)
if (!implementation || !path || !command) {
  console.error('usage: node-wrapper.mjs <dotenv|dotenv-expand> <file> <command> [args...]')
  process.exit(2)
}

if (implementation !== 'dotenv' && implementation !== 'dotenv-expand') {
  console.error(`unknown implementation: ${implementation}`)
  process.exit(2)
}
const parsed = dotenv.parse(readFileSync(path))
const loaded = implementation === 'dotenv-expand'
  ? (await import('dotenv-expand')).expand({ parsed, processEnv: {} }).parsed
  : parsed

const child = spawnSync(command, args, { env: { ...process.env, ...loaded }, stdio: 'inherit' })
if (child.error) throw child.error
process.exit(child.status ?? 1)
