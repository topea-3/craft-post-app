#!/usr/bin/env node
/**
 * Derive the next SemVer from the latest git tag (vX.Y.Z).
 *
 * Usage:
 *   node next-version.mjs <major|minor|patch> [--write]
 *
 * --write  Updates package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml
 *
 * Prints JSON: { current, next, tag, bump }
 */
import { execSync } from 'node:child_process'
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const bump = (process.argv[2] || '').toLowerCase()
const write = process.argv.includes('--write')

if (!['major', 'minor', 'patch'].includes(bump)) {
  console.error('Usage: node next-version.mjs <major|minor|patch> [--write]')
  process.exit(1)
}

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..', '..')

function latestTagVersion() {
  try {
    const out = execSync('git tag -l "v*" --sort=-v:refname', {
      cwd: root,
      encoding: 'utf8',
    })
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
  } catch {
    return '0.0.0'
  }
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

function writeVersions(next) {
  const pkgPath = join(root, 'package.json')
  const pkg = JSON.parse(readFileSync(pkgPath, 'utf8'))
  pkg.version = next
  writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`)

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

const current = latestTagVersion()
const next = bumpVersion(current, bump)
const tag = `v${next}`

if (write) writeVersions(next)

process.stdout.write(
  `${JSON.stringify({ current, next, tag, bump }, null, 2)}\n`,
)
