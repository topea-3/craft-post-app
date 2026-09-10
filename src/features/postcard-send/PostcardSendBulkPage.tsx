import { useEffect, useMemo, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useLocation, useNavigate } from 'react-router-dom'
import { formatLocalDate, isFutureLocalDate } from '../../lib/date'
import {
  formatDisplayName,
  fromAddressEntryDto,
  type AddressEntryDto,
} from '../address/types'
import { AddressEntrySelectDialog } from '../sender/AddressEntrySelectDialog'
import type { AddressEntryListItem } from '../address/types'
import { mapPostcardSendInvokeError } from './messages'
import type { BulkSendLocationState, PostcardType } from './types'
import { POSTCARD_TYPE_OPTIONS } from './types'

const MAX_SELECTION = 200
const MAX_MEMO_LENGTH = 1000

type SelectedRow = {
  id: string
  displayName: string
  senderEntryId: string | null
  senderLabel: string | null
  linkOk: boolean
}

export function PostcardSendBulkPage() {
  const navigate = useNavigate()
  const location = useLocation()
  const state = (location.state ?? null) as BulkSendLocationState | null
  const fromStatusTab = Boolean(state?.fromStatusTab)

  const [sentOn, setSentOn] = useState(formatLocalDate(new Date()))
  const [postcardType, setPostcardType] = useState<PostcardType>(
    state?.postcardType === 'mochu' ? 'mochu' : 'nenga',
  )
  const [memo, setMemo] = useState('')
  const [rows, setRows] = useState<SelectedRow[]>([])
  const [dialogOpen, setDialogOpen] = useState(false)
  const [formError, setFormError] = useState<string | null>(null)
  const [isSubmitting, setSubmitting] = useState(false)
  const submittingRef = useRef(false)
  const seededRef = useRef(false)

  useEffect(() => {
    if (seededRef.current) return
    const ids = state?.addressEntryIds ?? []
    if (ids.length === 0) return
    seededRef.current = true
    ;(async () => {
      const next: SelectedRow[] = []
      for (const id of ids.slice(0, MAX_SELECTION)) {
        try {
          const dto = await invoke<AddressEntryDto>('get_address_entry', { id })
          if (dto.archived) continue
          const entry = fromAddressEntryDto(dto)
          const displayName = formatDisplayName(entry.primaryName, entry.coRecipients)
          const senderId = await invoke<string | null>('get_sender_id_by_address_entry_id', {
            addressEntryId: id,
          })
          let senderLabel: string | null = null
          if (senderId) {
            try {
              const sender = await invoke<{ label: string; archived: boolean }>('get_sender_entry', {
                id: senderId,
              })
              if (!sender.archived) {
                senderLabel = sender.label
              }
            } catch {
              /* ignore */
            }
          }
          next.push({
            id,
            displayName,
            senderEntryId: senderLabel ? senderId : null,
            senderLabel,
            linkOk: Boolean(senderLabel),
          })
        } catch {
          /* skip missing */
        }
      }
      setRows(next)
    })()
  }, [state])

  const hasUnlinked = useMemo(() => rows.some((r) => !r.linkOk), [rows])
  const canSubmit = rows.length > 0 && !hasUnlinked && !isSubmitting

  const addAddress = async (item: AddressEntryListItem) => {
    setDialogOpen(false)
    if (rows.some((r) => r.id === item.id)) return
    if (rows.length >= MAX_SELECTION) {
      setFormError(`宛名は最大 ${MAX_SELECTION} 件までです。`)
      return
    }
    const displayName = formatDisplayName(item.primaryName, item.coRecipients)
    let senderEntryId: string | null = null
    let senderLabel: string | null = null
    try {
      const senderId = await invoke<string | null>('get_sender_id_by_address_entry_id', {
        addressEntryId: item.id,
      })
      if (senderId) {
        const sender = await invoke<{ label: string; archived: boolean }>('get_sender_entry', {
          id: senderId,
        })
        if (!sender.archived) {
          senderEntryId = senderId
          senderLabel = sender.label
        }
      }
    } catch {
      /* ignore */
    }
    setRows((prev) => [
      ...prev,
      {
        id: item.id,
        displayName,
        senderEntryId,
        senderLabel,
        linkOk: Boolean(senderLabel),
      },
    ])
    setFormError(null)
  }

  const removeRow = (id: string) => {
    setRows((prev) => prev.filter((r) => r.id !== id))
  }

  const handleCancel = () => {
    if (rows.length > 0 || memo.trim()) {
      const confirmed = window.confirm('入力内容を破棄して戻りますか？')
      if (!confirmed) return
    }
    navigate(fromStatusTab ? '/sends?tab=status' : '/sends')
  }

  const handleSubmit = async () => {
    if (submittingRef.current) return
    setFormError(null)

    if (!sentOn.trim()) {
      setFormError('送付日を入力してください。')
      return
    }
    if (isFutureLocalDate(sentOn)) {
      setFormError('送付日に未来の日付は指定できません。')
      return
    }
    if (rows.length === 0) {
      setFormError('宛名を1件以上選択してください。')
      return
    }
    if (hasUnlinked) {
      setFormError('差出人が紐づいていない宛名があります。除外するか紐付けてから登録してください。')
      return
    }
    if (Array.from(memo).length > MAX_MEMO_LENGTH) {
      setFormError(`メモは${MAX_MEMO_LENGTH}文字以内で入力してください。`)
      return
    }

    submittingRef.current = true
    setSubmitting(true)
    try {
      await invoke<{ ids: string[] }>('create_postcard_sends_manual_batch', {
        input: {
          postcard_type: postcardType,
          sent_on: sentOn,
          memo: memo.trim() || null,
          items: rows.map((r) => ({
            address_entry_id: r.id,
            sender_entry_id: r.senderEntryId,
          })),
        },
      })
      navigate('/sends')
    } catch (e) {
      console.error(e)
      setFormError(mapPostcardSendInvokeError(e))
      submittingRef.current = false
      setSubmitting(false)
    }
  }

  return (
    <div className="address-form-container">
      <header className="address-form-header">
        <h1>送付履歴一括登録</h1>
        <button type="button" className="link-button" onClick={handleCancel} disabled={isSubmitting}>
          キャンセル
        </button>
      </header>

      {formError ? <p className="address-form-error">{formError}</p> : null}
      {hasUnlinked ? (
        <p className="address-form-error">
          差出人が未紐付けの宛名があります。登録ボタンは無効です。
        </p>
      ) : null}

      <fieldset className="address-form-section" disabled={isSubmitting}>
        <legend>共通設定</legend>
        <label className="address-form-label">
          送付日
          <input type="date" value={sentOn} onChange={(e) => setSentOn(e.target.value)} />
        </label>
        <label className="address-form-label">
          種別
          <select
            value={postcardType}
            onChange={(e) => setPostcardType(e.target.value as PostcardType)}
          >
            {POSTCARD_TYPE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        <label className="address-form-label">
          メモ（全件共通）
          <textarea value={memo} onChange={(e) => setMemo(e.target.value)} rows={3} />
        </label>
      </fieldset>

      <fieldset className="address-form-section" disabled={isSubmitting}>
        <legend>宛名（{rows.length}/{MAX_SELECTION}）</legend>
        <button type="button" className="address-list-filter-toggle" onClick={() => setDialogOpen(true)}>
          宛名を追加
        </button>
        {rows.length === 0 ? (
          <p>宛名が選択されていません。</p>
        ) : (
          <table className="address-list-table">
            <thead>
              <tr>
                <th>宛名</th>
                <th>差出人</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.id}>
                  <td>{row.displayName}</td>
                  <td>{row.linkOk ? row.senderLabel : '未紐付け'}</td>
                  <td>
                    <button type="button" className="link-button" onClick={() => removeRow(row.id)}>
                      除外
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </fieldset>

      <div className="address-form-actions">
        <button type="button" className="link-button" onClick={handleCancel} disabled={isSubmitting}>
          キャンセル
        </button>
        <button
          type="button"
          className="address-list-create-button"
          disabled={!canSubmit}
          onClick={() => void handleSubmit()}
        >
          {isSubmitting ? '登録中…' : '一括登録'}
        </button>
      </div>

      <AddressEntrySelectDialog
        isOpen={dialogOpen}
        excludeIds={rows.map((r) => r.id)}
        onClose={() => setDialogOpen(false)}
        onSelect={(item) => {
          void addAddress(item)
        }}
      />
    </div>
  )
}
