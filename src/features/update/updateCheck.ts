import { getVersion } from '@tauri-apps/api/app'
import { check, type DownloadEvent } from '@tauri-apps/plugin-updater'

export const UPDATE_CHECK_TIMEOUT_MS = 30_000

export type UpdatePackage = {
  version: string
  body?: string | null
  downloadAndInstall: (onEvent?: (event: DownloadEvent) => void) => Promise<void>
}

export type UpdateCheckResult =
  | { kind: 'dev' }
  | { kind: 'failed' }
  | { kind: 'up-to-date'; currentVersion: string | null }
  | { kind: 'available'; update: UpdatePackage; currentVersion: string | null }

let inFlight: Promise<UpdateCheckResult> | null = null

export function resetUpdateCheckForTests(): void {
  inFlight = null
}

export function checkForUpdate(options?: { enabled?: boolean }): Promise<UpdateCheckResult> {
  const enabled = options?.enabled ?? import.meta.env.PROD
  if (!enabled) {
    return Promise.resolve({ kind: 'dev' })
  }
  if (inFlight) return inFlight
  inFlight = runCheck().finally(() => {
    inFlight = null
  })
  return inFlight
}

async function runCheck(): Promise<UpdateCheckResult> {
  const currentVersion = await readCurrentVersion()
  try {
    const update = await check({ timeout: UPDATE_CHECK_TIMEOUT_MS })
    if (!update) return { kind: 'up-to-date', currentVersion }
    return { kind: 'available', update, currentVersion }
  } catch (error) {
    console.error('checkForUpdate failed:', error)
    return { kind: 'failed' }
  }
}

async function readCurrentVersion(): Promise<string | null> {
  try {
    return await getVersion()
  } catch {
    return null
  }
}
