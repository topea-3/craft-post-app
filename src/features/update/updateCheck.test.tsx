import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { UpdateChecker } from './UpdateChecker'
import { checkForUpdate, resetUpdateCheckForTests, type UpdatePackage } from './updateCheck'

vi.mock('@tauri-apps/api/app', () => ({
  getVersion: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn(),
}))

const getVersionMock = vi.mocked(getVersion)
const relaunchMock = vi.mocked(relaunch)
const checkMock = vi.mocked(check)

function availableUpdate(body: string | null = '修正しました'): UpdatePackage {
  return {
    version: '0.2.0',
    body,
    downloadAndInstall: vi.fn().mockResolvedValue(undefined),
  }
}

describe('checkForUpdate', () => {
  beforeEach(() => {
    resetUpdateCheckForTests()
    checkMock.mockReset()
    getVersionMock.mockReset()
    getVersionMock.mockResolvedValue('0.1.1')
  })

  it('does not call the plugin when the build is not production', async () => {
    await expect(checkForUpdate({ enabled: false })).resolves.toEqual({ kind: 'dev' })
    expect(checkMock).not.toHaveBeenCalled()
  })

  it('returns up to date when the plugin resolves null', async () => {
    checkMock.mockResolvedValue(null)
    await expect(checkForUpdate({ enabled: true })).resolves.toEqual({
      kind: 'up-to-date',
      currentVersion: '0.1.1',
    })
    expect(checkMock).toHaveBeenCalledWith({ timeout: 30_000 })
  })

  it('omits the current version when getVersion fails', async () => {
    getVersionMock.mockRejectedValue(new Error('unavailable'))
    checkMock.mockResolvedValue(null)
    await expect(checkForUpdate({ enabled: true })).resolves.toEqual({
      kind: 'up-to-date',
      currentVersion: null,
    })
  })

  it('returns failed when the plugin rejects', async () => {
    checkMock.mockRejectedValue(new Error('offline'))
    await expect(checkForUpdate({ enabled: true })).resolves.toEqual({ kind: 'failed' })
  })

  it('shares one in-flight check', async () => {
    let resolveCheck: (value: null) => void = () => {}
    checkMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveCheck = resolve
        }),
    )

    const first = checkForUpdate({ enabled: true })
    const second = checkForUpdate({ enabled: true })
    await waitFor(() => expect(checkMock).toHaveBeenCalledTimes(1))
    resolveCheck(null)
    await expect(first).resolves.toMatchObject({ kind: 'up-to-date' })
    await expect(second).resolves.toMatchObject({ kind: 'up-to-date' })

    checkMock.mockResolvedValue(null)
    await checkForUpdate({ enabled: true })
    expect(checkMock).toHaveBeenCalledTimes(2)
  })
})

describe('UpdateChecker', () => {
  beforeEach(() => {
    resetUpdateCheckForTests()
    checkMock.mockReset()
    getVersionMock.mockReset()
    relaunchMock.mockReset()
    getVersionMock.mockResolvedValue('0.1.1')
    relaunchMock.mockResolvedValue()
  })

  afterEach(() => {
    document.querySelectorAll('.app-layout').forEach((node) => node.remove())
  })

  it('shows the update dialog on startup when a newer version exists', async () => {
    checkMock.mockResolvedValue(availableUpdate('ノート') as never)
    render(<UpdateChecker enabled />)
    expect(await screen.findByRole('dialog', { name: 'アップデート' })).toBeInTheDocument()
    expect(screen.getByText('バージョン 0.2.0 が利用できます。（現在 0.1.1）')).toBeInTheDocument()
    expect(screen.getByText('ノート')).toBeInTheDocument()
  })

  it('stays quiet on startup when there is no update', async () => {
    checkMock.mockResolvedValue(null)
    render(<UpdateChecker enabled />)
    await waitFor(() => expect(checkMock).toHaveBeenCalled())
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('stays quiet on startup when the check fails', async () => {
    checkMock.mockRejectedValue(new Error('offline'))
    render(<UpdateChecker enabled />)
    await waitFor(() => expect(checkMock).toHaveBeenCalled())
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('reports the current version from a manual check when already up to date', async () => {
    const user = userEvent.setup()
    checkMock.mockResolvedValue(null)
    render(<UpdateChecker enabled />)
    await user.click(screen.getByRole('button', { name: '更新を確認' }))
    expect(await screen.findByText('お使いのバージョンは最新です。（0.1.1）')).toBeInTheDocument()
  })

  it('opens the dialog and the sidebar status when a manual check finds an update', async () => {
    const user = userEvent.setup()
    checkMock.mockResolvedValue(availableUpdate() as never)
    render(<UpdateChecker enabled />)
    await user.click(await screen.findByRole('button', { name: '更新を確認' }))
    expect(await screen.findByRole('dialog', { name: 'アップデート' })).toBeInTheDocument()
    expect(screen.getByText('新しいバージョンがあります')).toBeInTheDocument()
  })

  it('reports a manual check failure', async () => {
    const user = userEvent.setup()
    checkMock.mockRejectedValue(new Error('offline'))
    render(<UpdateChecker enabled />)
    await user.click(screen.getByRole('button', { name: '更新を確認' }))
    expect(await screen.findByText('更新を確認できませんでした。')).toBeInTheDocument()
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('does not call the plugin from a development build and explains that', async () => {
    const user = userEvent.setup()
    render(<UpdateChecker enabled={false} />)
    await user.click(screen.getByRole('button', { name: '更新を確認' }))
    expect(await screen.findByText('開発中は更新を確認できません。')).toBeInTheDocument()
    expect(checkMock).not.toHaveBeenCalled()
  })

  it('closes the dialog without installing when the user waits', async () => {
    const user = userEvent.setup()
    const update = availableUpdate()
    checkMock.mockResolvedValue(update as never)
    render(<UpdateChecker enabled />)
    await screen.findByRole('dialog', { name: 'アップデート' })
    await user.click(screen.getByRole('button', { name: 'あとで' }))
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    expect(update.downloadAndInstall).not.toHaveBeenCalled()
  })

  it('downloads and relaunches when the user installs now', async () => {
    const user = userEvent.setup()
    let resolveDownload: () => void = () => {}
    const update = availableUpdate()
    update.downloadAndInstall = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          resolveDownload = resolve
        }),
    )
    checkMock.mockResolvedValue(update as never)
    render(<UpdateChecker enabled />)
    await user.click(await screen.findByRole('button', { name: '今すぐ更新' }))
    expect(screen.getByRole('button', { name: '今すぐ更新' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'あとで' })).toBeDisabled()
    resolveDownload()
    await waitFor(() => expect(relaunchMock).toHaveBeenCalledTimes(1))
    expect(update.downloadAndInstall).toHaveBeenCalledTimes(1)
  })

  it('keeps the dialog open when installation fails', async () => {
    const user = userEvent.setup()
    const update = availableUpdate()
    update.downloadAndInstall = vi.fn().mockRejectedValue(new Error('signature'))
    checkMock.mockResolvedValue(update as never)
    render(<UpdateChecker enabled />)
    await user.click(await screen.findByRole('button', { name: '今すぐ更新' }))
    expect(
      await screen.findByText('アップデートをインストールできませんでした。あとでもう一度試せます。'),
    ).toBeInTheDocument()
    expect(screen.getByRole('dialog', { name: 'アップデート' })).toBeInTheDocument()
    expect(relaunchMock).not.toHaveBeenCalled()
    expect(screen.getByRole('button', { name: '今すぐ更新' })).toBeEnabled()
  })

  it('checks once when a manual request joins the startup check', async () => {
    const user = userEvent.setup()
    let resolveCheck: (value: null) => void = () => {}
    checkMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveCheck = resolve
        }),
    )
    render(<UpdateChecker enabled />)
    await waitFor(() => expect(checkMock).toHaveBeenCalledTimes(1))
    await user.click(screen.getByRole('button', { name: '更新を確認' }))
    expect(screen.getByRole('button', { name: '確認しています…' })).toBeDisabled()
    expect(checkMock).toHaveBeenCalledTimes(1)
    resolveCheck(null)
    expect(await screen.findByText('お使いのバージョンは最新です。（0.1.1）')).toBeInTheDocument()
  })

  it('waits to show the dialog while the layout is inert', async () => {
    const layout = document.createElement('div')
    layout.className = 'app-layout'
    layout.setAttribute('inert', '')
    document.body.appendChild(layout)
    checkMock.mockResolvedValue(availableUpdate() as never)
    render(<UpdateChecker enabled />, { container: layout })
    await waitFor(() => expect(checkMock).toHaveBeenCalled())
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    layout.removeAttribute('inert')
    expect(await screen.findByRole('dialog', { name: 'アップデート' })).toBeInTheDocument()
  })

  it('renders release notes as text', async () => {
    checkMock.mockResolvedValue(availableUpdate('<img src="x" alt="note">') as never)
    render(<UpdateChecker enabled />)
    expect(await screen.findByText('<img src="x" alt="note">')).toBeInTheDocument()
    expect(screen.queryByRole('img')).not.toBeInTheDocument()
  })
})
