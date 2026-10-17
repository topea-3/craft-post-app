export const RESTORE_NOTICE_KEY = 'craftPostRestoreNotice'
export const RESTORE_NOTICE = 'バックアップから復元しました。'

export type BackupInspection = {
  createdAt: string
  appVersion: string
  schemaVersion: number
  encrypted: boolean
  byteSize: number
}

export type BackupErrorBody = {
  code: string
  message: string
}

export function defaultBackupFileName(now: Date): string {
  const pad = (value: number) => String(value).padStart(2, '0')
  return `craft-post-backup-${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}.cpbak`
}

export function passwordsMatch(password: string, confirm: string): boolean {
  return password.normalize('NFC') === confirm.normalize('NFC')
}

export function parseBackupError(error: unknown): BackupErrorBody {
  const text = errorText(error)
  try {
    const parsed = JSON.parse(text) as Partial<BackupErrorBody>
    if (typeof parsed.code === 'string' && typeof parsed.message === 'string') {
      return { code: parsed.code, message: parsed.message }
    }
  } catch {
    // 平文の Validation エラーなど
  }
  return { code: 'io', message: text || 'ファイルを読み書きできませんでした。' }
}

export function prepareRestoreReload(): void {
  sessionStorage.clear()
  sessionStorage.setItem(RESTORE_NOTICE_KEY, 'ok')
}

function errorText(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return ''
}
