import { useEffect, useState } from 'react'
import { createPortal } from 'react-dom'
import { relaunch } from '@tauri-apps/plugin-process'
import { Button } from '../../components/ui/Button'
import {
  checkForUpdate,
  type UpdateCheckResult,
  type UpdatePackage,
} from './updateCheck'

type Props = {
  enabled?: boolean
}

type PendingUpdate = Extract<UpdateCheckResult, { kind: 'available' }>

function layoutIsInteractive(): boolean {
  const layout = document.querySelector('.app-layout')
  return layout == null || !layout.hasAttribute('inert')
}

function latestMessage(currentVersion: string | null): string {
  if (!currentVersion) return 'お使いのバージョンは最新です。'
  return `お使いのバージョンは最新です。（${currentVersion}）`
}

function versionLine(update: UpdatePackage, currentVersion: string | null): string {
  if (!currentVersion) return `バージョン ${update.version} が利用できます。`
  return `バージョン ${update.version} が利用できます。（現在 ${currentVersion}）`
}

export function UpdateChecker({ enabled = import.meta.env.PROD }: Props) {
  const [interactive, setInteractive] = useState(layoutIsInteractive)
  const [checking, setChecking] = useState(false)
  const [manualStatus, setManualStatus] = useState<string | null>(null)
  const [pending, setPending] = useState<PendingUpdate | null>(null)
  const [installing, setInstalling] = useState(false)
  const [progressLabel, setProgressLabel] = useState<string | null>(null)
  const [installError, setInstallError] = useState<string | null>(null)

  useEffect(() => {
    const layout = document.querySelector('.app-layout')
    if (!layout) return
    const observer = new MutationObserver(() => {
      setInteractive(!layout.hasAttribute('inert'))
    })
    observer.observe(layout, { attributes: true, attributeFilter: ['inert'] })
    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    if (!enabled) return
    let cancelled = false
    void checkForUpdate({ enabled: true }).then((result) => {
      if (cancelled) return
      if (result.kind === 'available') setPending(result)
    })
    return () => {
      cancelled = true
    }
  }, [enabled])

  const onManual = async () => {
    setChecking(true)
    setManualStatus(null)
    try {
      const result = await checkForUpdate({ enabled })
      if (result.kind === 'dev') {
        setManualStatus('開発中は更新を確認できません。')
        return
      }
      if (result.kind === 'failed') {
        setManualStatus('更新を確認できませんでした。')
        return
      }
      if (result.kind === 'up-to-date') {
        setManualStatus(latestMessage(result.currentVersion))
        return
      }
      setManualStatus('新しいバージョンがあります')
      setInstallError(null)
      setProgressLabel(null)
      setPending(result)
    } finally {
      setChecking(false)
    }
  }

  const onLater = () => {
    setPending(null)
    setInstallError(null)
    setProgressLabel(null)
  }

  const onInstall = async () => {
    if (!pending || installing || !layoutIsInteractive()) return
    setInstalling(true)
    setInstallError(null)
    let downloaded = 0
    let total: number | null = null
    try {
      await pending.update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          total = event.data.contentLength ?? null
          downloaded = 0
          setProgressLabel(total == null ? 'ダウンロードしています' : `${downloaded} / ${total}`)
          return
        }
        if (event.event === 'Progress') {
          downloaded += event.data.chunkLength
          setProgressLabel(total == null ? 'ダウンロードしています' : `${downloaded} / ${total}`)
        }
      })
      await relaunch()
    } catch (error) {
      console.error('update install failed:', error)
      setInstallError('アップデートをインストールできませんでした。あとでもう一度試せます。')
      setInstalling(false)
      setProgressLabel(null)
    }
  }

  const notes = pending?.update.body?.trim() ? pending.update.body : null
  const dialog =
    pending && interactive
      ? createPortal(
          <div
            className="dialog-overlay"
            role="dialog"
            aria-modal="true"
            aria-label="アップデート"
          >
            <div className="dialog-panel update-dialog-panel">
              <h2 className="dialog-title">アップデート</h2>
              <div className="dialog-body">
                <p>{versionLine(pending.update, pending.currentVersion)}</p>
                {notes ? <p className="update-notes">{notes}</p> : null}
                <p>インストールを始めると、アプリはいったん終了します。</p>
                {progressLabel ? <p className="update-progress">{progressLabel}</p> : null}
                {installError ? <p>{installError}</p> : null}
                <div className="update-dialog-actions">
                  <Button variant="primary" disabled={installing} onClick={() => void onInstall()}>
                    今すぐ更新
                  </Button>
                  <Button disabled={installing} onClick={onLater}>
                    あとで
                  </Button>
                </div>
              </div>
            </div>
          </div>,
          document.body,
        )
      : null

  return (
    <div className="update-check">
      <Button disabled={checking} onClick={() => void onManual()}>
        {checking ? '確認しています…' : '更新を確認'}
      </Button>
      <p className="update-check-status" aria-live="polite">
        {manualStatus}
      </p>
      {dialog}
    </div>
  )
}
