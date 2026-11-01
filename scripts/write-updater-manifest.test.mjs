import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { describe, it } from 'node:test'
import assert from 'node:assert/strict'
import { buildUpdaterManifest } from './write-updater-manifest.mjs'

const scriptPath = fileURLToPath(new URL('./write-updater-manifest.mjs', import.meta.url))

const base = {
  version: '0.2.0',
  notes: 'リリースノート',
  signature: 'sig-body',
  assetName: 'Craft Post_0.2.0_x64-setup.exe',
  tag: 'v0.2.0',
  repo: 'topea-3/craft-post-app',
  pubDate: '2026-11-01T00:00:00Z',
}

describe('buildUpdaterManifest', () => {
  it('encodes spaces in the asset name and keeps only windows-x86_64', () => {
    const manifest = buildUpdaterManifest(base)
    assert.deepEqual(Object.keys(manifest.platforms), ['windows-x86_64'])
    assert.equal(
      manifest.platforms['windows-x86_64'].url,
      'https://github.com/topea-3/craft-post-app/releases/download/v0.2.0/Craft%20Post_0.2.0_x64-setup.exe',
    )
  })

  it('trims the signature', () => {
    const manifest = buildUpdaterManifest({ ...base, signature: '  abc \n' })
    assert.equal(manifest.platforms['windows-x86_64'].signature, 'abc')
  })

  it('rejects an empty signature, an empty file name, and a non-semver version', () => {
    assert.throws(() => buildUpdaterManifest({ ...base, signature: ' \n' }), /signature is empty/)
    assert.throws(() => buildUpdaterManifest({ ...base, assetName: '  ' }), /asset name is empty/)
    assert.throws(() => buildUpdaterManifest({ ...base, version: 'v0.2.0' }), /SemVer/)
  })
})

describe('write-updater-manifest cli', () => {
  it('writes latest.json', () => {
    const dir = mkdtempSync(join(tmpdir(), 'updater-manifest-'))
    try {
      const signature = join(dir, 'app.exe.sig')
      const notes = join(dir, 'notes.txt')
      const output = join(dir, 'latest.json')
      writeFileSync(signature, '  from-file \n')
      writeFileSync(notes, 'ノート')
      const result = spawnSync(
        process.execPath,
        [
          scriptPath,
          '--version',
          '0.2.0',
          '--tag',
          'v0.2.0',
          '--notes-file',
          notes,
          '--signature-file',
          signature,
          '--asset-name',
          'Craft Post_0.2.0_x64-setup.exe',
          '--repo',
          'topea-3/craft-post-app',
          '--pub-date',
          '2026-11-01T00:00:00Z',
          '--output',
          output,
        ],
        { encoding: 'utf8' },
      )
      assert.equal(result.status, 0, result.stderr)
      const written = JSON.parse(readFileSync(output, 'utf8'))
      assert.equal(written.platforms['windows-x86_64'].signature, 'from-file')
      assert.equal(written.notes, 'ノート')
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('exits non-zero when the signature is empty', () => {
    const dir = mkdtempSync(join(tmpdir(), 'updater-manifest-'))
    try {
      const signature = join(dir, 'app.exe.sig')
      const notes = join(dir, 'notes.txt')
      writeFileSync(signature, '   ')
      writeFileSync(notes, '')
      const result = spawnSync(
        process.execPath,
        [
          scriptPath,
          '--version',
          '0.2.0',
          '--tag',
          'v0.2.0',
          '--notes-file',
          notes,
          '--signature-file',
          signature,
          '--asset-name',
          'app.exe',
          '--repo',
          'topea-3/craft-post-app',
          '--pub-date',
          '2026-11-01T00:00:00Z',
          '--output',
          join(dir, 'latest.json'),
        ],
        { encoding: 'utf8' },
      )
      assert.notEqual(result.status, 0)
      assert.match(result.stderr, /signature is empty/)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })
})
