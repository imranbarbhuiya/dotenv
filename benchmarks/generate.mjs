import { writeFileSync } from 'node:fs'

const count = Number(process.argv[2] || 10000)
let output = '# generated benchmark fixture\n'
for (let i = 0; i < count; i++) {
  output += i > 0 && i % 4 === 0
    ? `KEY_${i}=prefix-$KEY_${i - 1}-${i} # expansion\n`
    : `KEY_${i}=value-${i} # comment\n`
}
output += 'MULTILINE="first\\nsecond"\nFINAL_URL=https://${KEY_1}/${KEY_2:-fallback}\n'
writeFileSync(new URL('./fixture.env', import.meta.url), output)
