// Emisor offline. La clave privada nunca se genera en el proyecto.
import { generateKeyPairSync, createPrivateKey, createPublicKey, sign, verify } from 'node:crypto'
import { existsSync, mkdirSync, readFileSync, writeFileSync, realpathSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const project = realpathSync(fileURLToPath(new URL('../..', import.meta.url)))
const [action, privatePath, request] = process.argv.slice(2)
if (!privatePath || !path.isAbsolute(privatePath)) throw Error('Indica una ruta privada absoluta fuera del proyecto')
const parent = path.dirname(privatePath)
const inside = dir => { const relative = path.relative(project, dir); return !relative.startsWith('..') && !path.isAbsolute(relative) }
if (inside(path.resolve(parent))) throw Error('La clave privada debe quedar fuera del proyecto')
if (action === 'crear') {
  if (existsSync(privatePath) || existsSync(parent)) throw Error('Usa una carpeta de emisor nueva; no se sobrescriben claves ni permisos existentes')
  mkdirSync(parent, { recursive: true, mode: 0o700 })
  if (inside(realpathSync(parent))) throw Error('La carpeta real pertenece al proyecto')
  if (process.platform === 'win32') {
    const identity = spawnSync('powershell', ['-NoProfile', '-Command', '[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value'], { encoding: 'utf8', windowsHide: true })
    if (identity.status !== 0 || !/^S-1-\d+(?:-\d+)+$/.test(identity.stdout.trim())) throw Error('No se pudo obtener el propietario Windows')
    const acl = spawnSync('icacls', [parent, '/inheritance:r', '/grant:r', `*${identity.stdout.trim()}:(OI)(CI)F`], { windowsHide: true })
    if (acl.status !== 0) throw Error('No se pudo restringir la carpeta privada')
  }
  const { privateKey, publicKey } = generateKeyPairSync('ed25519')
  writeFileSync(privatePath, privateKey.export({ format: 'pem', type: 'pkcs8' }), { flag: 'wx', mode: 0o600 })
  const publicHex = Buffer.from(publicKey.export({ format: 'jwk' }).x, 'base64url').toString('hex')
  const destination = fileURLToPath(new URL('../src-tauri/src/license_public_key.hex', import.meta.url))
  const existing = readFileSync(destination, 'utf8').trim()
  if (!/^0{64}$/.test(existing)) throw Error('Ya hay una clave pública configurada; no se cambia automáticamente')
  writeFileSync(destination, publicHex + '\n')
  console.log('Par Ed25519 creado; clave privada retenida exclusivamente en la ruta solicitada.')
} else if (action === 'firmar') {
  if (!/^LOGYM1-[0-9a-f]{64}$/.test(request || '')) throw Error('Solicitud inválida')
  if (inside(realpathSync(parent))) throw Error('La carpeta privada pertenece al proyecto')
  const key = createPrivateKey(readFileSync(privatePath))
  if (key.asymmetricKeyType !== 'ed25519') throw Error('Se requiere Ed25519')
  const configured = readFileSync(fileURLToPath(new URL('../src-tauri/src/license_public_key.hex', import.meta.url)), 'utf8').trim()
  const publicKey = createPublicKey(key)
  if (Buffer.from(publicKey.export({ format: 'jwk' }).x, 'base64url').toString('hex') !== configured) throw Error('El emisor no corresponde a la clave pública de la aplicación')
  const signature = sign(null, Buffer.from(request), key)
  if (!verify(null, Buffer.from(request), publicKey, signature)) throw Error('Falló la verificación local')
  console.log(request + ':' + signature.toString('hex'))
} else throw Error('Acciones permitidas: crear o firmar')
