import { execFileSync } from 'node:child_process'
import { cpus, platform, release, arch, totalmem } from 'node:os'
import { writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, resolve } from 'node:path'

const [output, entries, runs, warmup] = process.argv.slice(2)
const command = (program, args = []) => execFileSync(program, args, { encoding: 'utf8' }).trim()
const repo = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const git = (args) => command('git', ['-C', repo, ...args])
const metadata = {
  generated_at: new Date().toISOString(),
  git_commit: git(['rev-parse', 'HEAD']),
  git_dirty: Boolean(git(['status', '--porcelain'])),
  platform: `${platform()} ${release()} ${arch()}`,
  cpu: cpus()[0]?.model ?? 'unknown', logical_cpus: cpus().length, memory_bytes: totalmem(),
  rustc: command('rustc', ['--version']), cargo: command('cargo', ['--version']),
  node: command('node', ['--version']), npm: command('npm', ['--version']),
  hyperfine: command('hyperfine', ['--version']), fixture_entries: Number(entries),
  measured_runs_per_candidate: Number(runs), warmup_runs_per_candidate: Number(warmup),
  workload: 'load literal-only env file, launch shared verifier child, validate every value',
}
writeFileSync(output, `${JSON.stringify(metadata, null, 2)}\n`)
