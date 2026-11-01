import { readFileSync, writeFileSync } from 'node:fs'
import { pathToFileURL } from 'node:url'

const SEMVER = /^\d+\.\d+\.\d+$/
const RFC3339 = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$/

export function buildUpdaterManifest({ version, notes, signature, assetName, tag, repo, pubDate }) {
  if (!SEMVER.test(version)) {
    throw new Error(`version must be SemVer X.Y.Z: ${version}`)
  }
  const trimmedSignature = signature.trim()
  if (!trimmedSignature) throw new Error('signature is empty')
  if (!assetName || !assetName.trim()) throw new Error('asset name is empty')
  if (!tag || !tag.trim()) throw new Error('tag is empty')
  if (!repo || !repo.trim()) throw new Error('repo is empty')
  if (!RFC3339.test(pubDate)) throw new Error(`pub_date must be RFC3339: ${pubDate}`)

  return {
    version,
    notes,
    pub_date: pubDate,
    platforms: {
      'windows-x86_64': {
        signature: trimmedSignature,
        url: `https://github.com/${repo}/releases/download/${tag}/${encodeURIComponent(assetName)}`,
      },
    },
  }
}

function readOption(argv, name) {
  const index = argv.indexOf(name)
  if (index === -1 || argv[index + 1] == null) {
    throw new Error(`missing ${name}`)
  }
  return argv[index + 1]
}

export function runUpdaterManifestCli(argv, io = { readFileSync, writeFileSync }) {
  const signature = io.readFileSync(readOption(argv, '--signature-file'), 'utf8')
  const notes = io.readFileSync(readOption(argv, '--notes-file'), 'utf8')
  const manifest = buildUpdaterManifest({
    version: readOption(argv, '--version'),
    notes,
    signature,
    assetName: readOption(argv, '--asset-name'),
    tag: readOption(argv, '--tag'),
    repo: readOption(argv, '--repo'),
    pubDate: readOption(argv, '--pub-date'),
  })
  io.writeFileSync(readOption(argv, '--output'), `${JSON.stringify(manifest, null, 2)}\n`)
}

function isDirectRun() {
  const entry = process.argv[1]
  if (!entry) return false
  return import.meta.url === pathToFileURL(entry).href
}

if (isDirectRun()) {
  try {
    runUpdaterManifestCli(process.argv.slice(2))
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error))
    process.exit(1)
  }
}
