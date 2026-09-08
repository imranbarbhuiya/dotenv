import { writeFileSync } from 'node:fs'

const count = Number(process.argv[2] || 1000)
if (!Number.isSafeInteger(count) || count < 1) throw new Error('entry count must be a positive integer')

let output = '# Portable literal-only fixture shared by every candidate.\n'
for (let i = 0; i < count; i++) output += `BENCH_KEY_${i}=value-${i}\n`
writeFileSync(new URL('./fixture.env', import.meta.url), output)
