// Auditor de empaquetado; nunca se importa desde src/ ni inicia la aplicación.
import assert from 'node:assert/strict'
import { existsSync, readdirSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const root = fileURLToPath(new URL('../', import.meta.url))
const native = path.join(root, 'src-tauri', 'src')
const read = file => readFileSync(file, 'utf8').replaceAll('\r\n', '\n')
const walk = dir => readdirSync(dir, { withFileTypes: true }).flatMap(entry => {
  const file = path.join(dir, entry.name)
  return entry.isDirectory() ? walk(file) : [file]
})

assert(!existsSync(path.join(native, 'test_support.rs')), 'Los fixtures deben estar en src/tests/')
for (const file of readdirSync(native).filter(name => name.endsWith('.rs'))) {
  const source = read(path.join(native, file))
  assert(!source.includes('#[test]'), `Test incrustado en ${file}`)
  const paths = [...source.matchAll(/#\[path = "tests\/[^"\n]+"\]/g)]
  for (const match of paths) {
    assert(source.slice(0, match.index).endsWith('#[cfg(test)]\n'), `Falta cfg(test) en ${file}`)
  }
}

for (const file of walk(path.join(root, 'src')).filter(file => /\.(js|vue)$/.test(file))) {
  for (const match of read(file).matchAll(/(?:\bfrom\s*|\bimport\s*\(?\s*)['"]([^'"]+)['"]/g)) {
    assert(!/(?:^|\/)(?:tests|\.artifacts|scripts)(?:\/|$)/.test(match[1]), `Import de tests en ${file}: ${match[1]}`)
  }
}

const config = JSON.parse(read(path.join(root, 'src-tauri', 'tauri.conf.json')))
assert.deepEqual(config.tauri.bundle.resources, ['bin/sgfplib.dll', 'bin/sgfpamx.dll', 'bin/sgwsqlib.dll', 'bin/sgfdusdax64.dll', 'bin/sgbledev.dll'], 'Revisar la whitelist de recursos Tauri')
assert.equal(path.resolve(root, 'src-tauri', config.build.distDir), path.join(root, 'dist'))
assert(!existsSync(path.join(root, 'public')), 'Si se agrega public/, auditar su contenido antes de publicar')
for (const file of walk(path.join(root, 'src-tauri', 'bin'))) {
  assert(!/(?:^|[\\/])(?:tests|docs|scripts|\.artifacts|coverage)(?:[\\/]|$)/.test(file), `Recurso de pruebas: ${file}`)
  assert(!/\.(?:rs|mjs|test\.js)$/.test(file), `Código de pruebas en recursos: ${file}`)
}

const markers = [
  'logym-test-', 'fixture.sqlite',
  'simulated_capture_timeout_invalid_read_extraction_retry_and_exact_template_size',
  'defecto_migracion_cliente_legacy_sin_columnas_de_fechas',
]
const assets = walk(path.join(root, 'dist'))
for (const file of assets) {
  assert(!/(?:^|[\\/])(?:tests|docs|scripts|\.artifacts|coverage)(?:[\\/]|$)/.test(file))
  const data = readFileSync(file)
  for (const marker of markers) assert(!data.includes(Buffer.from(marker)), `Fixture en asset productivo: ${file}`)
}
console.log(`PASS: módulos Rust condicionados, imports JS separados, ${assets.length} assets Vite y recursos Tauri sin tests.`)

if (process.argv.includes('--release')) {
  const release = path.join(root, 'src-tauri', 'target', 'release')
  const dependencies = read(path.join(release, 'logym.d')).replaceAll('\\', '/')
  assert(!dependencies.includes('/src/tests/'), 'Tests en las dependencias del binario release')
  assert(!dependencies.includes('test_support.rs'), 'Fixtures en las dependencias del binario release')
  const executable = readFileSync(path.join(release, process.platform === 'win32' ? 'logym.exe' : 'logym'))
  for (const marker of markers) {
    for (const encoding of ['utf8', 'utf16le']) assert(!executable.includes(Buffer.from(marker, encoding)), `Marcador de test en release: ${marker}`)
  }
  console.log('PASS: dependencias de rustc y marcadores del ejecutable release sin tests/fixtures.')
}
