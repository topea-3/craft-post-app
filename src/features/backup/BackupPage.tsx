import { useEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import {
  RESTORE_NOTICE,
  RESTORE_NOTICE_KEY,
  defaultBackupFileName,
  parseBackupError,
  passwordPolicyError,
  passwordsMatch,
  prepareRestoreReload,
  type BackupInspection,
} from './backupModel'

type BusyPhase = 'idle' | 'inspect' | 'working'

type BackupPageProps = {
  saveDialog?: (defaultPath: string) => Promise<string | null>
  openDialog?: () => Promise<string | null>
  reloadPage?: () => void
  now?: Date
}

function formatCreatedAt(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString('ja-JP')
}

function formatBytes(size: number): string {
  return `${size.toLocaleString('ja-JP')} バイト`
}

const BACKUP_FILTER = [{ name: 'Craft Post バックアップ (*.cpbak)', extensions: ['cpbak'] }]

async function defaultSaveDialog(defaultPath: string): Promise<string | null> {
  return save({ defaultPath, filters: BACKUP_FILTER })
}

async function defaultOpenDialog(): Promise<string | null> {
  const selected = await open({ multiple: false, filters: BACKUP_FILTER })
  return typeof selected === 'string' ? selected : null
}

export function BackupPage({
  saveDialog = defaultSaveDialog,
  openDialog = defaultOpenDialog,
  reloadPage = () => window.location.reload(),
  now,
}: BackupPageProps) {
  const [notice] = useState(() =>
    sessionStorage.getItem(RESTORE_NOTICE_KEY) === 'ok' ? RESTORE_NOTICE : null,
  )
  const [protect, setProtect] = useState(false)
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [saveMessage, setSaveMessage] = useState<string | null>(null)
  const [saveError, setSaveError] = useState<string | null>(null)
  const [sourcePath, setSourcePath] = useState<string | null>(null)
  const [inspection, setInspection] = useState<BackupInspection | null>(null)
  const [restorePassword, setRestorePassword] = useState('')
  const [needsPassword, setNeedsPassword] = useState(false)
  const [restoreError, setRestoreError] = useState<string | null>(null)
  const [restoreCode, setRestoreCode] = useState<string | null>(null)
  const [confirmed, setConfirmed] = useState(false)
  const [phase, setPhase] = useState<BusyPhase>('idle')
  const phaseRef = useRef<BusyPhase>('idle')
  const overlayRef = useRef<HTMLDivElement>(null)

  const setBusy = (next: BusyPhase) => {
    phaseRef.current = next
    setPhase(next)
  }

  useEffect(() => {
    if (!notice) return
    const timer = window.setTimeout(() => sessionStorage.removeItem(RESTORE_NOTICE_KEY), 0)
    return () => window.clearTimeout(timer)
  }, [notice])

  const busyLabel = phase === 'inspect' ? '確認しています' : phase === 'working' ? '処理しています' : null

  useEffect(() => {
    if (!busyLabel) return
    const layout = document.querySelector('.app-layout')
    layout?.setAttribute('inert', '')
    overlayRef.current?.focus()
    const trapKeys = (event: KeyboardEvent) => {
      if (event.key !== 'Tab' && event.key !== 'Escape') return
      event.preventDefault()
      overlayRef.current?.focus()
    }
    document.addEventListener('keydown', trapKeys)
    return () => {
      layout?.removeAttribute('inert')
      document.removeEventListener('keydown', trapKeys)
    }
  }, [busyLabel])

  const resetRestoreSelection = () => {
    setInspection(null)
    setRestorePassword('')
    setNeedsPassword(false)
    setRestoreError(null)
    setRestoreCode(null)
    setConfirmed(false)
  }

  const inspectFile = async (path: string, passwordValue: string | null) => {
    setBusy('inspect')
    try {
      const result = await invoke<BackupInspection>('inspect_backup', {
        sourcePath: path,
        password: passwordValue,
      })
      setInspection(result)
      setNeedsPassword(false)
      setRestoreError(null)
      setRestoreCode(null)
    } catch (error) {
      const parsed = parseBackupError(error)
      setInspection(null)
      setRestoreCode(parsed.code)
      setRestoreError(parsed.message)
      setNeedsPassword(parsed.code === 'password_required' || parsed.code === 'password_invalid')
      if (parsed.code === 'password_invalid') setRestorePassword('')
    } finally {
      setBusy('idle')
    }
  }

  const handleProtectChange = (enabled: boolean) => {
    setProtect(enabled)
    setSaveError(null)
    if (!enabled) {
      setPassword('')
      setConfirmPassword('')
    }
  }

  const handleSave = async () => {
    if (phaseRef.current !== 'idle') return
    setSaveError(null)
    setSaveMessage(null)
    if (protect && !passwordsMatch(password, confirmPassword)) {
      setSaveError('確認用パスワードが一致しません。')
      return
    }
    if (protect) {
      const policyError = passwordPolicyError(password)
      if (policyError) {
        setSaveError(policyError)
        return
      }
    }
    const selected = await saveDialog(defaultBackupFileName(now ?? new Date()))
    if (!selected || phaseRef.current !== 'idle') return
    setBusy('working')
    try {
      await invoke('create_backup', {
        destinationPath: selected,
        password: protect ? password.normalize('NFC') : null,
      })
      setSaveMessage(`バックアップを保存しました ${selected}`)
    } catch (error) {
      setSaveError(parseBackupError(error).message)
    } finally {
      setBusy('idle')
    }
  }

  const handlePick = async () => {
    if (phaseRef.current !== 'idle') return
    const selected = await openDialog()
    if (!selected || phaseRef.current !== 'idle') return
    resetRestoreSelection()
    setSourcePath(selected)
    await inspectFile(selected, null)
  }

  const handleInspectWithPassword = async () => {
    if (phaseRef.current !== 'idle' || !sourcePath) return
    await inspectFile(sourcePath, restorePassword.normalize('NFC'))
  }

  const handleRestore = async () => {
    if (phaseRef.current !== 'idle') return
    if (!sourcePath || !inspection || restoreCode === 'schema_too_new') return
    setBusy('working')
    try {
      await invoke('restore_backup', {
        sourcePath,
        password: inspection.encrypted ? restorePassword.normalize('NFC') : null,
      })
      prepareRestoreReload()
      reloadPage()
    } catch (error) {
      const parsed = parseBackupError(error)
      setRestoreCode(parsed.code)
      setRestoreError(parsed.message)
      setBusy('idle')
    }
  }

  return (
    <section className="backup-page">
      <h2 className="form-section-title">バックアップ</h2>
      {notice ? <p className="form-success-banner">{notice}</p> : null}

      <section className="form-section">
        <h3 className="form-section-title">バックアップを作成</h3>
        <p>住所録、差出人、送受信履歴、印刷レイアウトを 1 つのファイルに保存する</p>
        <label className="field-label">
          <span>
            <input
              type="checkbox"
              checked={protect}
              onChange={(event) => handleProtectChange(event.target.checked)}
            />
            パスワードで保護する
          </span>
        </label>
        {protect ? (
          <>
            <p>パスワードを忘れると、このバックアップからは復元できません。</p>
            <p>8 文字以上 128 文字以内で入力してください。</p>
            <label className="field-label">
              パスワード
              <input
                type="password"
                value={password}
                maxLength={128}
                aria-label="パスワード"
                onChange={(event) => setPassword(event.target.value)}
              />
            </label>
            <label className="field-label">
              パスワード（確認）
              <input
                type="password"
                value={confirmPassword}
                maxLength={128}
                aria-label="パスワード（確認）"
                onChange={(event) => setConfirmPassword(event.target.value)}
              />
            </label>
          </>
        ) : (
          <p>保護しないファイルには住所などの個人情報が平文で入ります。</p>
        )}
        {saveError ? <p className="form-error-banner">{saveError}</p> : null}
        {saveMessage ? <p>{saveMessage}</p> : null}
        <button
          type="button"
          className="btn btn-label btn-primary"
          disabled={phase !== 'idle'}
          onClick={() => void handleSave()}
        >
          バックアップを保存
        </button>
      </section>

      <section className="form-section">
        <h3 className="form-section-title">バックアップから復元</h3>
        <button
          type="button"
          className="btn btn-label btn-normal"
          disabled={phase !== 'idle'}
          onClick={() => void handlePick()}
        >
          バックアップファイルを選択
        </button>
        {sourcePath ? <p>選択中: {sourcePath}</p> : null}
        {needsPassword ? (
          <>
            <label className="field-label">
              パスワード
              <input
                type="password"
                value={restorePassword}
                aria-label="復元パスワード"
                onChange={(event) => setRestorePassword(event.target.value)}
              />
            </label>
            <button
              type="button"
              className="btn btn-label btn-normal"
              disabled={phase !== 'idle'}
              onClick={() => void handleInspectWithPassword()}
            >
              内容を確認
            </button>
          </>
        ) : null}
        {restoreError ? <p className="form-error-banner">{restoreError}</p> : null}
        {inspection ? (
          <>
            <dl className="backup-inspection">
              <div>
                <dt>作成日時</dt>
                <dd>{formatCreatedAt(inspection.createdAt)}</dd>
              </div>
              <div>
                <dt>アプリのバージョン</dt>
                <dd>{inspection.appVersion}</dd>
              </div>
              <div>
                <dt>スキーマ</dt>
                <dd>{inspection.schemaVersion}</dd>
              </div>
              <div>
                <dt>サイズ</dt>
                <dd>{formatBytes(inspection.byteSize)}</dd>
              </div>
              <div>
                <dt>保護</dt>
                <dd>{inspection.encrypted ? 'パスワード保護' : '保護なし'}</dd>
              </div>
            </dl>
            <p>
              現在の住所録・差出人・送受信履歴・印刷レイアウトは、このバックアップの内容で完全に置き換わります。置き換え後は元に戻せません。未保存の入力と、印刷画面の選択も破棄されます。必要なら先に現在のデータをバックアップしてください。
            </p>
            <label className="field-label">
              <span>
                <input
                  type="checkbox"
                  checked={confirmed}
                  aria-label="現在のデータをこのバックアップで置き換える"
                  onChange={(event) => setConfirmed(event.target.checked)}
                />
                現在のデータをこのバックアップで置き換える
              </span>
            </label>
            <button
              type="button"
              className="btn btn-label btn-primary"
              disabled={phase !== 'idle' || !confirmed}
              onClick={() => void handleRestore()}
            >
              復元する
            </button>
          </>
        ) : null}
      </section>

      {busyLabel
        ? createPortal(
            <div
              ref={overlayRef}
              tabIndex={-1}
              className="backup-block-overlay"
              role="alertdialog"
              aria-modal="true"
              aria-label={busyLabel}
            >
              <p>{busyLabel}</p>
            </div>,
            document.body,
          )
        : null}
    </section>
  )
}
