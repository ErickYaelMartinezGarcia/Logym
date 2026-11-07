import { spawnSync } from 'node:child_process'
import { mkdirSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const root = fileURLToPath(new URL('../', import.meta.url))
const output = path.join(root, '.artifacts', 'tests', 'latest')
mkdirSync(output, { recursive: true })
const defects = process.argv.includes('--defects')
const commands = [
  { name: 'cargo-check', bin: 'cargo', args: ['check', '--locked'], cwd: path.join(root, 'src-tauri') },
  { name: 'rust', bin: 'cargo', args: ['test', '--locked'], cwd: path.join(root, 'src-tauri') },
  { name: 'frontend', bin: process.execPath, args: ['--experimental-vm-modules', '--experimental-test-coverage', '--test-coverage-include=src/**', '--test', 'tests/frontend.test.mjs'], cwd: root },
]
if (defects) commands.push({ name: 'known-defects', bin: 'cargo', args: ['test', '--locked', 'defecto_', '--', '--nocapture'], cwd: path.join(root, 'src-tauri') })
const results = []
for (const command of commands) {
  console.log(`\n> ${command.bin} ${command.args.join(' ')}`)
  const started = new Date().toISOString()
  const result = spawnSync(command.bin, command.args, { cwd: command.cwd, encoding: 'utf8', maxBuffer: 20 * 1024 * 1024, windowsHide: true })
  const log = `${result.stdout || ''}${result.stderr || ''}${result.error ? `\n${result.error.stack}` : ''}`
  writeFileSync(path.join(output, `${command.name}.log`), log)
  process.stdout.write(log)
  results.push({ name: command.name, command: [command.bin, ...command.args], cwd: command.cwd,
    started, exitCode: result.status, status: result.status === 0 ? 'PASS' : 'FAIL',
    diagnosticSuite: command.name === 'known-defects', log: `${command.name}.log` })
}
writeFileSync(path.join(output, 'results.json'), JSON.stringify({ finished: new Date().toISOString(), results }, null, 2))
console.log(`\nResultados: ${output}`)
// Nunca ocultar el fallo de una prueba, ni siquiera de la suite diagnóstica.
process.exitCode = results.some(result => result.status === 'FAIL') ? 1 : 0
