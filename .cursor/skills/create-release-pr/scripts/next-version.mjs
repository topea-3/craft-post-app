#!/usr/bin/env node
/**
 * Derive the next SemVer from the latest git tag (vX.Y.Z), or write a fixed version.
 *
 * Usage:
 *   node next-version.mjs <major|minor|patch>
 *   node next-version.mjs --set <X.Y.Z>
 *
 * --set  Writes package.json / tauri.conf.json / Cargo.toml to the given version
 *        (does not re-derive from tags). Use after a successful derive in the same run.
 *
 * Prints JSON: { current, next, tag, bump } for bump mode,
 * or { next, tag, written: true } for --set mode.
 */
import { execSync } from 'node:child_process'
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..', '..')
const args = process.argv.slice(2)

function usage() {
  console.error(
    'Usage: node next-version.mjs <major|minor|patch>\n' +
      '       node next-version.mjs --set <X.Y.Z>',
  )
  process.exit(1)
}

function runGit(command) {
  try {
    return execSync(command, {
      cwd: root,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    })
  } catch (err) {
    const stderr = err.stderr?.toString?.() || err.message || String(err)
    console.error(`git command failed: ${command}`)
    console.error(stderr.trim())
    process.exit(1)
  }
}

function latestTagVersion() {
  const out = runGit('git tag -l "v*" --sort=-v:refname')
    .trim()
    .split(/\r?\n/)
    .filter(Boolean)
  if (out.length === 0) return '0.0.0'
  const m = out[0].match(/^v(\d+)\.(\d+)\.(\d+)$/)
  if (!m) {
    console.error(`Unrecognized latest tag: ${out[0]} (expected vX.Y.Z)`)
    process.exit(1)
  }
  return `${m[1]}.${m[2]}.${m[3]}`
}

function bumpVersion(current, kind) {
  const [maj, min, pat] = current.split('.').map((n) => Number(n))
  if ([maj, min, pat].some((n) => Number.isNaN(n))) {
    console.error(`Invalid current version: ${current}`)
    process.exit(1)
  }
  if (kind === 'major') return `${maj + 1}.0.0`
  if (kind === 'minor') return `${maj}.${min + 1}.0`
  return `${maj}.${min}.${pat + 1}`
}

function assertSemVer(version) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    console.error(`Invalid version (expected X.Y.Z): ${version}`)
    process.exit(1)
  }
}

function writeVersions(next) {
  assertSemVer(next)
  const pkgPath = join(root, 'package.json')
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'))
  pkg.version = next
  writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`)

  const lockPath = join(root, 'package-lock.json')
  try {
    const lock = JSON.parse(readFileSync(lockPath, 'utf8'))
    lock.version = next
    if (lock.packages && lock.packages['']) {
      lock.packages[''].version = next
    }
    writeFileSync(lockPath, `${JSON.stringify(lock, null, 2)}\n`)
  } catch (err) {
    if (err && err.code !== 'ENOENT') {
      console.error('Failed to update package-lock.json version')
      console.error(err.message || String(err))
      process.exit(1)
    }
  }

  const tauriPath = join(root, 'src-tauri', 'tauri.conf.json')
  const tauri = JSON.parse(readFileSync(tauriPath, 'utf8'))
  tauri.version = next
  writeFileSync(tauriPath, `${JSON.stringify(tauri, null, 2)}\n`)

  const cargoPath = join(root, 'src-tauri', 'Cargo.toml')
  const cargo = readFileSync(cargoPath, 'utf8')
  if (!/^version\s*=\s*"[^"]*"/m.test(cargo)) {
    console.error('Failed to find version in src-tauri/Cargo.toml')
    process.exit(1)
  }
  const updated = cargo.replace(
    /^version\s*=\s*"[^"]*"/m,
    `version = "${next}"`,
  )
  writeFileSync(cargoPath, updated)
}

if (args[0] === '--set') {
  const next = args[1]
  if (!next) usage()
  writeVersions(next)
  process.stdout.write(
    `${JSON.stringify({ next, tag: `v${next}`, written: true }, null, 2)}\n`,
  )
  process.exit(0)
}

const bump = (args[0] || '').toLowerCase()
if (!['major', 'minor', 'patch'].includes(bump)) usage()
if (args.includes('--write')) {
  console.error(
    'Error: --write is removed. Derive once, then use --set <X.Y.Z> to write.',
  )
  process.exit(1)
}

const current = latestTagVersion()
const next = bumpVersion(current, bump)
const tag = `v${next}`

process.stdout.write(
  `${JSON.stringify({ current, next, tag, bump }, null, 2)}\n`,
)
