import { writeFileSync } from 'node:fs'

const count = Number(process.argv[2] || 10000)
let output = '# generated benchmark fixture\n'
for (let i = 0; i < count; i++) output += `KEY_${i}=value-${i} # comment\n`
output += 'MULTILINE="first\\nsecond"\nBENCHMARK_TARGET=old\n'
writeFileSync(new URL('./fixture.env', import.meta.url), output)

