import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { BackupPage } from './BackupPage'
import { prepareRestoreReload } from './backupModel'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

const inspection = {
  createdAt: '2026-10-17T09:00:00+09:00',
  appVersion: '0.1.1',
  schemaVersion: 9,
  encrypted: false,
  byteSize: 4096,
}

describe('BackupPage', () => {
  beforeEach(() => {
    sessionStorage.clear()
    invokeMock.mockReset()
  })

  it('does not call create when the confirmation password differs, including before NFC', async () => {
    const user = userEvent.setup()
    const saveDialog = vi.fn()
    render(<BackupPage saveDialog={saveDialog} now={new Date(2026, 9, 17, 9, 0, 0)} />)

    await user.click(screen.getByRole('checkbox', { name: 'パスワードで保護する' }))
    await user.type(screen.getByLabelText('パスワード'), 'password')
    await user.type(screen.getByLabelText('パスワード（確認）'), 'password!')
    await user.click(screen.getByRole('button', { name: 'バックアップを保存' }))

    expect(screen.getByText('確認用パスワードが一致しません。')).toBeInTheDocument()
    expect(saveDialog).not.toHaveBeenCalled()
    expect(invokeMock).not.toHaveBeenCalled()
  })

  it('treats NFC-equivalent confirmation as a match and sends the normalized password', async () => {
    const user = userEvent.setup()
    const saveDialog = vi.fn().mockResolvedValue('D:/backup.cpbak')
    invokeMock.mockResolvedValue(inspection)
    render(<BackupPage saveDialog={saveDialog} now={new Date(2026, 9, 17, 9, 0, 1)} />)

    await user.click(screen.getByRole('checkbox', { name: 'パスワードで保護する' }))
    await user.type(screen.getByLabelText('パスワード'), '\u304C')
    await user.type(screen.getByLabelText('パスワード（確認）'), '\u304B\u3099')
    await user.click(screen.getByRole('button', { name: 'バックアップを保存' }))

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('create_backup', {
        destinationPath: 'D:/backup.cpbak',
        password: '\u304C',
      })
    })
  })

  it('sends null and drops hidden password fields when protection is turned off', async () => {
    const user = userEvent.setup()
    const saveDialog = vi.fn().mockResolvedValue('D:/plain.cpbak')
    invokeMock.mockResolvedValue(inspection)
    render(<BackupPage saveDialog={saveDialog} now={new Date(2026, 9, 17, 9, 0, 2)} />)

    await user.click(screen.getByRole('checkbox', { name: 'パスワードで保護する' }))
    await user.type(screen.getByLabelText('パスワード'), 'secret-pass')
    await user.type(screen.getByLabelText('パスワード（確認）'), 'secret-pass!')
    await user.click(screen.getByRole('button', { name: 'バックアップを保存' }))
    expect(screen.getByText('確認用パスワードが一致しません。')).toBeInTheDocument()
    await user.click(screen.getByRole('checkbox', { name: 'パスワードで保護する' }))
    expect(screen.queryByText('確認用パスワードが一致しません。')).not.toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'バックアップを保存' }))

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('create_backup', {
        destinationPath: 'D:/plain.cpbak',
        password: null,
      })
    })
  })

  it('blocks the screen with 確認しています while inspect is running', async () => {
    const user = userEvent.setup()
    let resolveInspect: (value: typeof inspection) => void = () => {}
    invokeMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveInspect = resolve
        }),
    )
    render(<BackupPage openDialog={async () => 'D:/a.cpbak'} />)

    await user.click(screen.getByRole('button', { name: 'バックアップファイルを選択' }))
    expect(await screen.findByRole('alertdialog', { name: '確認しています' })).toBeInTheDocument()
    resolveInspect(inspection)
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
    })
  })

  it('clears the password and confirmation checkbox when another file is chosen', async () => {
    const user = userEvent.setup()
    const openDialog = vi
      .fn()
      .mockResolvedValueOnce('D:/first.cpbak')
      .mockResolvedValueOnce('D:/second.cpbak')
    invokeMock
      .mockRejectedValueOnce(
        JSON.stringify({ code: 'password_required', message: 'このバックアップはパスワードで保護されています。' }),
      )
      .mockResolvedValueOnce({ ...inspection, encrypted: true })
      .mockResolvedValueOnce(inspection)
    render(<BackupPage openDialog={openDialog} />)

    await user.click(screen.getByRole('button', { name: 'バックアップファイルを選択' }))
    const password = await screen.findByLabelText('復元パスワード')
    await user.type(password, 'secret-pass')
    await user.click(screen.getByRole('button', { name: '内容を確認' }))
    await user.click(await screen.findByLabelText('現在のデータをこのバックアップで置き換える'))
    await user.click(screen.getByRole('button', { name: 'バックアップファイルを選択' }))

    await waitFor(() => {
      expect(invokeMock).toHaveBeenLastCalledWith('inspect_backup', {
        sourcePath: 'D:/second.cpbak',
        password: null,
      })
    })
    expect(screen.queryByLabelText('復元パスワード')).not.toBeInTheDocument()
    expect(screen.getByLabelText('現在のデータをこのバックアップで置き換える')).not.toBeChecked()
  })

  it('keeps the same file and allows another password after password_invalid', async () => {
    const user = userEvent.setup()
    invokeMock
      .mockRejectedValueOnce(
        JSON.stringify({ code: 'password_required', message: 'このバックアップはパスワードで保護されています。' }),
      )
      .mockRejectedValueOnce(
        JSON.stringify({ code: 'password_invalid', message: 'パスワードが違うか、ファイルが壊れています。' }),
      )
      .mockResolvedValueOnce({ ...inspection, encrypted: true })
    render(<BackupPage openDialog={async () => 'D:/locked.cpbak'} />)

    await user.click(screen.getByRole('button', { name: 'バックアップファイルを選択' }))
    const password = await screen.findByLabelText('復元パスワード')
    await user.type(password, 'wrong-password')
    await user.click(screen.getByRole('button', { name: '内容を確認' }))
    expect(await screen.findByText('パスワードが違うか、ファイルが壊れています。')).toBeInTheDocument()
    expect(screen.getByLabelText('復元パスワード')).toHaveValue('')
    expect(screen.getByText('選択中: D:/locked.cpbak')).toBeInTheDocument()

    await user.type(screen.getByLabelText('復元パスワード'), 'right-password')
    await user.click(screen.getByRole('button', { name: '内容を確認' }))
    expect(await screen.findByRole('button', { name: '復元する' })).toBeDisabled()
    expect(invokeMock).toHaveBeenLastCalledWith('inspect_backup', {
      sourcePath: 'D:/locked.cpbak',
      password: 'right-password',
    })
  })

  it('clears print session keys and leaves only the restore notice before reload', async () => {
    const user = userEvent.setup()
    sessionStorage.setItem('printJobDraft', '{"addressEntryIds":["a1"]}')
    sessionStorage.setItem('printPostcardType', 'nenga')
    const reloadPage = vi.fn()
    invokeMock.mockResolvedValueOnce(inspection)
    render(<BackupPage openDialog={async () => 'D:/ok.cpbak'} reloadPage={reloadPage} />)

    await user.click(screen.getByRole('button', { name: 'バックアップファイルを選択' }))
    await user.click(await screen.findByLabelText('現在のデータをこのバックアップで置き換える'))
    await user.click(screen.getByRole('button', { name: '復元する' }))

    await waitFor(() => expect(reloadPage).toHaveBeenCalled())
    expect(sessionStorage.getItem('printJobDraft')).toBeNull()
    expect(sessionStorage.getItem('printPostcardType')).toBeNull()
    expect(sessionStorage.getItem('craftPostRestoreNotice')).toBe('ok')
    expect(sessionStorage.length).toBe(1)
  })

  it('shows the restore notice once and then removes the key', async () => {
    sessionStorage.setItem('craftPostRestoreNotice', 'ok')
    render(<BackupPage />)
    expect(screen.getByText('バックアップから復元しました。')).toBeInTheDocument()
    await waitFor(() => expect(sessionStorage.getItem('craftPostRestoreNotice')).toBeNull())
  })
})

describe('prepareRestoreReload', () => {
  it('keeps only the restore notice', () => {
    sessionStorage.setItem('printJobDraft', 'draft')
    sessionStorage.setItem('printPostcardType', 'nenga')
    prepareRestoreReload()
    expect(sessionStorage.getItem('printJobDraft')).toBeNull()
    expect(sessionStorage.getItem('printPostcardType')).toBeNull()
    expect(sessionStorage.getItem('craftPostRestoreNotice')).toBe('ok')
  })
})
