import { useEffect, useMemo, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Link, useNavigate, useSearchParams } from 'react-router-dom'
import { PaginationControls } from '../../components/PaginationControls'
import { currentLocalYear } from '../../lib/date'
import { clampPage, totalPagesFor } from '../../lib/pagination'
import { POSTCARD_SEND_OPERATION_ERROR_MESSAGE } from './messages'
import {
  readPrintJobDraftAddressIds,
  replacePrintJobDraftAddressIds,
  syncPrintPostcardType,
} from './printHandoff'
import type { BulkSendLocationState, PostcardType, SendStatusFilter } from './types'
import {
  POSTCARD_SEND_SOURCE_OPTIONS,
  POSTCARD_TYPE_OPTIONS,
  MAX_SEARCH_KEYWORD_LENGTH,
  buildHistoryYearOptions,
  buildReceiptYearOptions,
  buildStatusYearOptions,
  formatMemoSnippet,
  formatSentOn,
  postcardTypeLabel,
  resolveAddressDisplayName,
  resolveSenderDisplayName,
  sourceLabel,
} from './types'
import { usePostcardSendList } from './usePostcardSendList'
import { useSendStatusList } from './useSendStatusList'

const PAGE_SIZE = 20
const MAX_SELECTION = 200

type Tab = 'history' | 'status'

export function PostcardSendManagePage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const tab: Tab = searchParams.get('tab') === 'status' ? 'status' : 'history'

  const setTab = (next: Tab) => {
    const params = new URLSearchParams(searchParams)
    if (next === 'status') {
      params.set('tab', 'status')
    } else {
      params.delete('tab')
    }
    setSearchParams(params, { replace: true })
  }

  return (
    <div className="address-list-container">
      <div className="address-list-header">
        <h1 className="address-list-title">送付履歴</h1>
        <div className="address-list-header-actions">
          {tab === 'history' ? (
            <>
              <Link to="/sends/new" className="address-list-create-button">
                新規作成
              </Link>
              <Link to="/sends/bulk" className="address-list-filter-toggle">
                一括登録
              </Link>
            </>
          ) : null}
        </div>
      </div>

      <div className="address-list-filter" style={{ marginBottom: '1rem' }}>
        <div role="tablist" aria-label="送付管理タブ" style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            type="button"
            role="tab"
            id="postcard-send-history-tab"
            aria-controls="postcard-send-history-panel"
            aria-selected={tab === 'history'}
            className={tab === 'history' ? 'address-list-create-button' : 'address-list-filter-toggle'}
            onClick={() => setTab('history')}
          >
            履歴
          </button>
          <button
            type="button"
            role="tab"
            id="postcard-send-status-tab"
            aria-controls="postcard-send-status-panel"
            aria-selected={tab === 'status'}
            className={tab === 'status' ? 'address-list-create-button' : 'address-list-filter-toggle'}
            onClick={() => setTab('status')}
          >
            送付状況
          </button>
        </div>
      </div>

      <div
        role="tabpanel"
        hidden={tab !== 'history'}
        id="postcard-send-history-panel"
        aria-labelledby="postcard-send-history-tab"
      >
        <HistoryTab active={tab === 'history'} />
      </div>
      <div
        role="tabpanel"
        hidden={tab !== 'status'}
        id="postcard-send-status-panel"
        aria-labelledby="postcard-send-status-tab"
      >
        <StatusTab active={tab === 'status'} />
      </div>
    </div>
  )
}

function HistoryTab({ active }: { active: boolean }) {
  const navigate = useNavigate()
  const [isFilterOpen, setIsFilterOpen] = useState(false)
  const [searchText, setSearchText] = useState('')
  const [year, setYear] = useState('')
  const [postcardType, setPostcardType] = useState('')
  const [source, setSource] = useState('')
  const [page, setPage] = useState(1)
  const [availableYears, setAvailableYears] = useState<number[]>([])
  const [yearsError, setYearsError] = useState<string | null>(null)
  const [yearsReloadToken, setYearsReloadToken] = useState(0)
  const [deletingId, setDeletingId] = useState<string | null>(null)
  const isDeletingRef = useRef(false)
  const cancelledRef = useRef(false)

  const { items, total, isLoading, error, settledSearchText, isDebouncing, reload } =
    usePostcardSendList({
      searchText,
      year,
      postcardType,
      source,
      page,
      pageSize: PAGE_SIZE,
      onPageChange: setPage,
    })

  useEffect(() => {
    cancelledRef.current = false
    return () => {
      cancelledRef.current = true
    }
  }, [])

  useEffect(() => {
    if (!active) return
    let cancelled = false
    ;(async () => {
      try {
        const years = await invoke<number[]>('list_postcard_send_years')
        if (cancelled) return
        setAvailableYears(years)
        setYearsError(null)
      } catch (e) {
        console.error('Failed to load postcard send years:', e)
        if (!cancelled) {
          setYearsError(POSTCARD_SEND_OPERATION_ERROR_MESSAGE)
        }
      }
    })()
    return () => {
      cancelled = true
    }
  }, [active, yearsReloadToken])

  const yearOptions = useMemo(() => {
    const options = buildHistoryYearOptions(availableYears)
    if (year && !options.some((o) => o.value === year)) {
      return [...options, { value: year, label: year }]
    }
    return options
  }, [availableYears, year])

  const totalPages = totalPagesFor(total, PAGE_SIZE)
  const currentPage = clampPage(page, total, PAGE_SIZE)
  const isFiltering =
    settledSearchText.trim().length > 0 || year !== '' || postcardType !== '' || source !== ''
  const showSettledEmpty = !isDebouncing && !isLoading
  const isNoData = showSettledEmpty && !isFiltering && total === 0
  const isNoSearchResult = showSettledEmpty && isFiltering && total === 0
  const hasItems = total > 0
  const isBusy = deletingId !== null

  const handleDelete = (id: string) => {
    if (isDeletingRef.current) return
    const confirmed = window.confirm('この送付履歴を削除しますか？一覧からは非表示になります。')
    if (!confirmed) return
    isDeletingRef.current = true
    setDeletingId(id)
    ;(async () => {
      try {
        await invoke('delete_postcard_send', { id })
        if (cancelledRef.current) return
        reload()
        setYearsReloadToken((t) => t + 1)
      } catch (deleteError) {
        if (cancelledRef.current) return
        console.error('Failed to delete postcard send:', deleteError)
        alert(POSTCARD_SEND_OPERATION_ERROR_MESSAGE)
      } finally {
        isDeletingRef.current = false
        if (!cancelledRef.current) {
          setDeletingId(null)
        }
      }
    })()
  }

  return (
    <>
      <div className="address-list-header-actions" style={{ marginBottom: '0.75rem' }}>
        <button
          type="button"
          className="address-list-filter-toggle"
          onClick={() => setIsFilterOpen((open) => !open)}
          disabled={isBusy}
        >
          フィルタ
        </button>
      </div>

      {isFilterOpen ? (
        <div className="address-list-filter">
          <label className="address-list-filter-label">
            <span>検索</span>
            <input
              type="text"
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              placeholder="宛名・差出人・メモで検索"
              className="address-list-filter-input"
              maxLength={MAX_SEARCH_KEYWORD_LENGTH}
              disabled={isBusy}
            />
          </label>
          <label className="address-list-filter-label">
            <span>送付年</span>
            <select
              value={year}
              onChange={(e) => {
                setYear(e.target.value)
                setPage(1)
              }}
              disabled={isBusy}
              aria-label="送付年"
            >
              {yearOptions.map((option) => (
                <option key={option.value || 'all'} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          <label className="address-list-filter-label">
            <span>種別</span>
            <select
              value={postcardType}
              onChange={(e) => {
                setPostcardType(e.target.value)
                setPage(1)
              }}
              disabled={isBusy}
              aria-label="種別"
            >
              <option value="">全種別</option>
              {POSTCARD_TYPE_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          <label className="address-list-filter-label">
            <span>登録経路</span>
            <select
              value={source}
              onChange={(e) => {
                setSource(e.target.value)
                setPage(1)
              }}
              disabled={isBusy}
              aria-label="登録経路"
            >
              <option value="">すべて</option>
              {POSTCARD_SEND_SOURCE_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          <button
            type="button"
            className="address-list-filter-toggle"
            onClick={() => {
              setSearchText('')
              setYear('')
              setPostcardType('')
              setSource('')
              setPage(1)
            }}
            disabled={isBusy}
          >
            条件クリア
          </button>
        </div>
      ) : null}

      {yearsError ? (
        <p className="address-list-error">
          {yearsError}{' '}
          <button
            type="button"
            className="link-button"
            onClick={() => setYearsReloadToken((t) => t + 1)}
          >
            再試行
          </button>
        </p>
      ) : null}
      {error ? <p className="address-list-error">{error}</p> : null}
      {isLoading ? <p>読み込み中…</p> : null}

      {!isLoading && isNoData ? (
        <div className="address-list-empty">
          <p>まだ送付履歴がありません。</p>
          <p>
            <Link to="/sends/new">新規作成</Link>
            {' / '}
            <Link to="/print/select">宛名印刷</Link>
          </p>
        </div>
      ) : null}

      {!isLoading && isNoSearchResult ? (
        <div className="address-list-empty">
          <p>該当する送付履歴が見つかりませんでした。</p>
          <button
            type="button"
            className="link-button"
            onClick={() => {
              setSearchText('')
              setYear('')
              setPostcardType('')
              setSource('')
              setPage(1)
            }}
          >
            条件クリア
          </button>
        </div>
      ) : null}

      {hasItems ? (
        <>
          <table className="address-list-table">
            <thead>
              <tr>
                <th>送付日</th>
                <th>種別</th>
                <th>宛名</th>
                <th>差出人</th>
                <th>経路</th>
                <th>メモ</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => {
                const memo = formatMemoSnippet(item.memo)
                return (
                  <tr
                    key={item.id}
                    onClick={() => navigate(`/sends/${item.id}`)}
                    style={{ cursor: 'pointer' }}
                  >
                    <td>{formatSentOn(item.sentOn)}</td>
                    <td>{postcardTypeLabel(item.postcardType)}</td>
                    <td>
                      <strong>{resolveAddressDisplayName(item)}</strong>
                    </td>
                    <td>{resolveSenderDisplayName(item)}</td>
                    <td>{sourceLabel(item.source)}</td>
                    <td>
                      {memo.text}
                      {memo.truncated ? '…' : ''}
                    </td>
                    <td onClick={(e) => e.stopPropagation()}>
                      <button
                        type="button"
                        className="link-button"
                        disabled={isBusy}
                        onClick={() => navigate(`/sends/${item.id}/edit`)}
                      >
                        編集
                      </button>
                      <button
                        type="button"
                        className="link-button"
                        disabled={isBusy}
                        onClick={() => handleDelete(item.id)}
                      >
                        削除
                      </button>
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
      ) : null}
    </>
  )
}

function StatusTab({ active }: { active: boolean }) {
  const navigate = useNavigate()
  const thisYear = currentLocalYear()
  const [searchText, setSearchText] = useState('')
  const [year, setYear] = useState(thisYear)
  const [postcardType, setPostcardType] = useState('')
  const [status, setStatus] = useState<SendStatusFilter>('unsent')
  const [limitByReceipt, setLimitByReceipt] = useState(true)
  const [receiptYear, setReceiptYear] = useState(thisYear - 1)
  const [receiptYearManual, setReceiptYearManual] = useState(false)
  const [page, setPage] = useState(1)
  const [sendYears, setSendYears] = useState<number[]>([])
  const [receiptYears, setReceiptYears] = useState<number[]>([])
  const [yearsError, setYearsError] = useState<string | null>(null)
  const [yearsReloadToken, setYearsReloadToken] = useState(0)
  const [selectedIds, setSelectedIds] = useState<string[]>([])
  const [selectionError, setSelectionError] = useState<string | null>(null)

  const apiReceiptYear = limitByReceipt ? receiptYear : null

  const { items, total, isLoading, error, settledSearchText, isDebouncing } = useSendStatusList({
    searchText,
    year,
    postcardType,
    status,
    receiptYear: apiReceiptYear,
    page,
    pageSize: PAGE_SIZE,
    onPageChange: setPage,
  })

  useEffect(() => {
    if (!active) return
    let cancelled = false
    ;(async () => {
      setYearsError(null)
      let sendOk = false
      let receiptOk = false
      try {
        const sYears = await invoke<number[]>('list_postcard_send_years')
        if (!cancelled) {
          setSendYears(sYears)
          sendOk = true
        }
      } catch (e) {
        console.error('Failed to load send years for send status:', e)
      }
      try {
        const rYears = await invoke<number[]>('list_postcard_receipt_years')
        if (!cancelled) {
          setReceiptYears(rYears)
          receiptOk = true
        }
      } catch (e) {
        console.error('Failed to load receipt years for send status:', e)
      }
      if (!cancelled && (!sendOk || !receiptOk)) {
        setYearsError(POSTCARD_SEND_OPERATION_ERROR_MESSAGE)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [active, yearsReloadToken])

  const yearOptions = useMemo(
    () => buildStatusYearOptions(sendYears, thisYear, year),
    [sendYears, thisYear, year],
  )
  const receiptYearOptions = useMemo(
    () => buildReceiptYearOptions(year, receiptYears, receiptYear),
    [year, receiptYears, receiptYear],
  )

  const clearSelection = () => {
    setSelectedIds([])
    setSelectionError(null)
  }

  const handleYearChange = (nextYear: number) => {
    setYear(nextYear)
    setPage(1)
    clearSelection()
    if (!receiptYearManual) {
      setReceiptYear(nextYear - 1)
    }
  }

  const totalPages = totalPagesFor(total, PAGE_SIZE)
  const currentPage = clampPage(page, total, PAGE_SIZE)
  const hasKeyword = settledSearchText.trim().length > 0
  const showSettledEmpty = !isDebouncing && !isLoading
  const isEmpty = showSettledEmpty && total === 0

  const toggleSelect = (id: string) => {
    setSelectionError(null)
    if (selectedIds.includes(id)) {
      setSelectedIds((prev) => prev.filter((x) => x !== id))
      return
    }
    if (selectedIds.length >= MAX_SELECTION) {
      setSelectionError(`選択できる宛名は最大 ${MAX_SELECTION} 件です。`)
      return
    }
    setSelectedIds((prev) => [...prev, id])
  }

  const handlePrint = () => {
    if (selectedIds.length === 0) return
    const existing = readPrintJobDraftAddressIds()
    if (existing.length > 0) {
      const confirmed = window.confirm(
        '進行中の印刷下書きがあります。選択した宛名で置き換えて印刷画面へ進みますか？',
      )
      if (!confirmed) return
    }
    replacePrintJobDraftAddressIds(selectedIds)
    clearSelection()
    if (postcardType === 'nenga' || postcardType === 'mochu') {
      syncPrintPostcardType(postcardType)
    }
    navigate('/print/select')
  }

  const handleBulk = () => {
    if (selectedIds.length === 0) return
    const state: BulkSendLocationState = {
      addressEntryIds: selectedIds,
      postcardType:
        postcardType === 'nenga' || postcardType === 'mochu'
          ? (postcardType as PostcardType)
          : null,
      fromStatusTab: true,
    }
    navigate('/sends/bulk', { state })
  }

  return (
    <>
      <div className="address-list-filter">
        <label className="address-list-filter-label">
          <span>対象年</span>
          <select
            value={String(year)}
            onChange={(e) => handleYearChange(Number(e.target.value))}
            aria-label="対象年"
          >
            {yearOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        <label className="address-list-filter-label">
          <span>種別</span>
          <select
            value={postcardType}
            onChange={(e) => {
              setPostcardType(e.target.value)
              setPage(1)
              clearSelection()
            }}
            aria-label="種別"
          >
            <option value="">すべて</option>
            {POSTCARD_TYPE_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        <div className="address-list-filter-label" role="group" aria-label="表示">
          <span>表示</span>
          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <button
              type="button"
              className={
                status === 'unsent' ? 'address-list-create-button' : 'address-list-filter-toggle'
              }
              onClick={() => {
                setStatus('unsent')
                setPage(1)
                clearSelection()
              }}
            >
              送っていない
            </button>
            <button
              type="button"
              className={
                status === 'sent' ? 'address-list-create-button' : 'address-list-filter-toggle'
              }
              onClick={() => {
                setStatus('sent')
                setPage(1)
                clearSelection()
              }}
            >
              送った
            </button>
          </div>
        </div>
        <label className="address-list-filter-label">
          <span>
            <input
              type="checkbox"
              checked={limitByReceipt}
              onChange={(e) => {
                setLimitByReceipt(e.target.checked)
                setPage(1)
                clearSelection()
              }}
            />{' '}
            受取履歴のある相手に限定
          </span>
        </label>
        {limitByReceipt ? (
          <label className="address-list-filter-label">
            <span>受取年</span>
            <select
              value={String(receiptYear)}
              onChange={(e) => {
                setReceiptYearManual(true)
                setReceiptYear(Number(e.target.value))
                setPage(1)
                clearSelection()
              }}
              aria-label="受取年"
            >
              {receiptYearOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
        ) : null}
        <label className="address-list-filter-label">
          <span>検索</span>
          <input
            type="text"
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            placeholder="住所録の表示名で検索"
            className="address-list-filter-input"
            maxLength={MAX_SEARCH_KEYWORD_LENGTH}
          />
        </label>
      </div>

      {status === 'unsent' && selectedIds.length > 0 ? (
        <div className="address-list-header-actions" style={{ marginBottom: '0.75rem' }}>
          <span>{selectedIds.length} 件選択中</span>
          <button type="button" className="address-list-create-button" onClick={handlePrint}>
            選択して印刷
          </button>
          <button type="button" className="address-list-filter-toggle" onClick={handleBulk}>
            選択して一括登録
          </button>
          <button type="button" className="link-button" onClick={clearSelection}>
            選択解除
          </button>
        </div>
      ) : null}

      {selectionError ? <p className="address-list-error">{selectionError}</p> : null}
      {yearsError ? (
        <p className="address-list-error">
          {yearsError}{' '}
          <button
            type="button"
            className="link-button"
            onClick={() => setYearsReloadToken((t) => t + 1)}
          >
            再試行
          </button>
        </p>
      ) : null}
      {error ? <p className="address-list-error">{error}</p> : null}
      {isLoading || isDebouncing ? <p>読み込み中…</p> : null}

      {isEmpty && hasKeyword ? (
        <div className="address-list-empty">
          <p>該当する相手が見つかりませんでした。</p>
          <button type="button" className="link-button" onClick={() => setSearchText('')}>
            検索クリア
          </button>
        </div>
      ) : null}

      {isEmpty && !hasKeyword && status === 'unsent' ? (
        <div className="address-list-empty">
          <p>この条件では未送付の相手はいません。</p>
        </div>
      ) : null}

      {isEmpty && !hasKeyword && status === 'sent' ? (
        <div className="address-list-empty">
          <p>この条件では送付記録がありません。</p>
        </div>
      ) : null}

      {!isLoading && !isDebouncing && total > 0 ? (
        <>
          <table className="address-list-table">
            <thead>
              <tr>
                {status === 'unsent' ? <th>選択</th> : null}
                <th>宛名</th>
                <th>住所</th>
                <th>最終送付日</th>
                <th>送付件数</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => (
                <tr key={item.addressEntryId}>
                  {status === 'unsent' ? (
                    <td>
                      <input
                        type="checkbox"
                        checked={selectedIds.includes(item.addressEntryId)}
                        onChange={() => toggleSelect(item.addressEntryId)}
                        aria-label={`${item.displayName} を選択`}
                      />
                    </td>
                  ) : null}
                  <td>
                    <strong>{item.displayName}</strong>
                  </td>
                  <td>{item.addressSummary}</td>
                  <td>{item.lastSentOn ? formatSentOn(item.lastSentOn) : '—'}</td>
                  <td>{item.sendCount}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <PaginationControls
            currentPage={currentPage}
            totalPages={totalPages}
            onPrev={() => setPage((prev) => Math.max(1, prev - 1))}
            onNext={() => setPage((prev) => Math.min(totalPages, prev + 1))}
          />
        </>
      ) : null}
    </>
  )
}
