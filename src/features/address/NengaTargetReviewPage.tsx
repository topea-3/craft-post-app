import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'
import { PaginationControls } from '../../components/PaginationControls'
import { Button } from '../../components/ui/Button'
import {
  ADDRESS_OPERATION_ERROR_MESSAGE,
  NENGA_TARGET_REVIEW_CONFIRM_MESSAGE,
  NENGA_TARGET_REVIEW_EMPTY_MESSAGE,
  NENGA_TARGET_REVIEW_LOAD_FAILED_MESSAGE,
  NENGA_TARGET_REVIEW_NONE_UPDATED_MESSAGE,
  NENGA_TARGET_REVIEW_SKIPPED_MESSAGE,
  NENGA_TARGET_REVIEW_UPDATED_MESSAGE,
  NENGA_TARGET_YEAR_RANGE_MESSAGE,
} from './messages'
import {
  formatAddressSingleLine,
  formatDisplayName,
  formatPostalCode,
  type AddressDto,
  type PersonNameDto,
  type PersonNameForm,
} from './types'

const PAGE_SIZE = 20
const YEAR_MIN = 1900
const YEAR_MAX = 2100

type ReviewItemDto = {
  id: string
  primary_name: PersonNameDto
  co_recipients: PersonNameDto[]
  honorific: string
  postal_code: string
  address: AddressDto
}

type DisableResult = {
  updated_count: number
  skipped_count: number
}

function localCalendarYear(): number {
  return new Date().getFullYear()
}

function parseReviewYear(text: string): number | null {
  const trimmed = text.trim()
  if (!/^\d+$/.test(trimmed)) return null
  const year = Number(trimmed)
  if (!Number.isInteger(year) || year < YEAR_MIN || year > YEAR_MAX) return null
  return year
}

function toPersonName(dto: PersonNameDto): PersonNameForm {
  return {
    last: dto.last,
    first: dto.first,
    kanaLast: dto.kana_last ?? undefined,
    kanaFirst: dto.kana_first ?? undefined,
  }
}

export function NengaTargetReviewPage() {
  const navigate = useNavigate()
  const [yearText, setYearText] = useState(() => String(localCalendarYear()))
  const [reloadKey, setReloadKey] = useState(0)
  const [items, setItems] = useState<ReviewItemDto[]>([])
  const [checked, setChecked] = useState<Set<string>>(() => new Set())
  const [page, setPage] = useState(1)
  const [isLoading, setIsLoading] = useState(true)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [loadError, setLoadError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  const year = parseReviewYear(yearText)

  useEffect(() => {
    if (year == null) return
    let cancelled = false
    ;(async () => {
      try {
        const rows = await invoke<ReviewItemDto[]>('list_nenga_target_review', { year })
        if (cancelled) return
        setItems(rows)
        setChecked(new Set(rows.map((row) => row.id)))
        setLoadError(null)
      } catch (error) {
        if (cancelled) return
        console.error('list_nenga_target_review failed:', error)
        setItems([])
        setChecked(new Set())
        setLoadError(NENGA_TARGET_REVIEW_LOAD_FAILED_MESSAGE)
      } finally {
        if (!cancelled) setIsLoading(false)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [year, reloadKey])

  const totalPages = Math.max(1, Math.ceil(items.length / PAGE_SIZE))
  const currentPage = Math.min(page, totalPages)
  const pageItems = useMemo(() => {
    const start = (currentPage - 1) * PAGE_SIZE
    return items.slice(start, start + PAGE_SIZE)
  }, [items, currentPage])

  const allChecked = items.length > 0 && items.every((item) => checked.has(item.id))
  const checkedCount = checked.size

  const handleYearChange = (value: string) => {
    setYearText(value)
    setPage(1)
    setNotice(null)
    if (parseReviewYear(value) != null) {
      setIsLoading(true)
      setLoadError(null)
    } else {
      setIsLoading(false)
      setItems([])
      setChecked(new Set())
      setLoadError(null)
    }
  }

  const toggleAll = () => {
    setChecked(allChecked ? new Set() : new Set(items.map((item) => item.id)))
  }

  const toggleOne = (id: string) => {
    setChecked((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  const handleDisable = async () => {
    if (checkedCount === 0 || year == null || isSubmitting) return
    const confirmed = window.confirm(NENGA_TARGET_REVIEW_CONFIRM_MESSAGE(checkedCount))
    if (!confirmed) return

    setIsSubmitting(true)
    setNotice(null)
    try {
      const result = await invoke<DisableResult>('disable_nenga_send_targets', {
        ids: [...checked],
      })
      if (result.updated_count === 0) {
        setNotice(NENGA_TARGET_REVIEW_NONE_UPDATED_MESSAGE)
      } else {
        const parts = [NENGA_TARGET_REVIEW_UPDATED_MESSAGE(result.updated_count)]
        if (result.skipped_count > 0) {
          parts.push(NENGA_TARGET_REVIEW_SKIPPED_MESSAGE(result.skipped_count))
        }
        setNotice(parts.join(''))
      }
      setIsLoading(true)
      setReloadKey((key) => key + 1)
    } catch (error) {
      console.error('disable_nenga_send_targets failed:', error)
      setNotice(ADDRESS_OPERATION_ERROR_MESSAGE)
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="address-list-container">
      <div className="address-list-header">
        <h1 className="address-list-title">年賀状送付対象の見直し</h1>
        <div className="address-list-header-actions">
          <Button type="button" onClick={() => navigate('/addresses')}>
            一覧へ戻る
          </Button>
        </div>
      </div>

      <p className="field-helper">
        選択した年の年賀状を受け取っておらず、その前年の喪中もない、年賀状送付対象の宛名です。
      </p>

      <label className="address-list-filter-label">
        <span>年賀状年</span>
        <input
          type="number"
          min={YEAR_MIN}
          max={YEAR_MAX}
          value={yearText}
          onChange={(e) => handleYearChange(e.target.value)}
        />
      </label>

      {year == null && <p className="address-list-error">{NENGA_TARGET_YEAR_RANGE_MESSAGE}</p>}
      {notice && <p role="status">{notice}</p>}
      {loadError && <p className="address-list-error">{loadError}</p>}
      {isLoading && year != null && <p className="address-list-loading">読み込み中です…</p>}

      {!isLoading && year != null && !loadError && items.length === 0 && (
        <p className="address-list-empty">{NENGA_TARGET_REVIEW_EMPTY_MESSAGE}</p>
      )}

      {!isLoading && year != null && !loadError && items.length > 0 && (
        <>
          <table className="address-list-table" aria-label="年賀状送付対象の見直し">
            <thead>
              <tr>
                <th>
                  <label>
                    <input
                      type="checkbox"
                      checked={allChecked}
                      onChange={toggleAll}
                      aria-label="読み込んだ候補をまとめて年賀状対象外にする"
                    />
                    年賀状対象外にする
                  </label>
                </th>
                <th>氏名</th>
                <th>住所</th>
              </tr>
            </thead>
            <tbody>
              {pageItems.map((item) => {
                const primary = toPersonName(item.primary_name)
                const co = item.co_recipients.map(toPersonName)
                const name = formatDisplayName(primary, co)
                const withHonorific =
                  item.honorific && item.honorific !== 'なし' ? `${name} ${item.honorific}` : name
                return (
                  <tr key={item.id}>
                    <td>
                      <input
                        type="checkbox"
                        checked={checked.has(item.id)}
                        onChange={() => toggleOne(item.id)}
                        aria-label={`${withHonorific} を年賀状対象外にする`}
                      />
                    </td>
                    <td>
                      {withHonorific}
                      <div>{formatPostalCode(item.postal_code)}</div>
                    </td>
                    <td>
                      {formatAddressSingleLine({
                        postalCode: item.postal_code,
                        prefecture: item.address.prefecture,
                        city: item.address.city,
                        street: item.address.street,
                        building: item.address.building ?? undefined,
                      })}
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
          <PaginationControls
            currentPage={currentPage}
            totalPages={totalPages}
            onPrev={() => setPage((prev) => Math.max(1, prev - 1))}
            onNext={() => setPage((prev) => Math.min(totalPages, prev + 1))}
          />
        </>
      )}

      <div className="form-footer">
        <Button
          type="button"
          variant="primary"
          onClick={handleDisable}
          disabled={checkedCount === 0 || year == null || isSubmitting || isLoading || loadError != null}
        >
          年賀状送付対象外にする
        </Button>
      </div>
    </div>
  )
}
