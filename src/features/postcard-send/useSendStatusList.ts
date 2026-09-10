import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { clampPage } from '../../lib/pagination'
import type { PostcardType, SendStatusFilter, SendStatusItem, SendStatusItemDto } from './types'
import { POSTCARD_SEND_OPERATION_ERROR_MESSAGE } from './messages'
import { fromSendStatusItemDto } from './types'

const SEARCH_DEBOUNCE_MS = 300

type UseSendStatusListParams = {
  searchText: string
  year: number
  postcardType: string
  status: SendStatusFilter
  /** null = 受取限定 OFF（API に receiptYear を送らない） */
  receiptYear: number | null
  page: number
  pageSize: number
  onPageChange?: (page: number) => void
}

type UseSendStatusListResult = {
  items: SendStatusItem[]
  total: number
  isLoading: boolean
  error: string | null
  settledSearchText: string
  reload: () => void
}

export function useSendStatusList(params: UseSendStatusListParams): UseSendStatusListResult {
  const { searchText, year, postcardType, status, receiptYear, page, pageSize, onPageChange } =
    params
  const [debouncedSearchText, setDebouncedSearchText] = useState(searchText)
  const [items, setItems] = useState<SendStatusItem[]>([])
  const [total, setTotal] = useState(0)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [reloadToken, setReloadToken] = useState(0)

  const debouncedRef = useRef(debouncedSearchText)
  debouncedRef.current = debouncedSearchText
  const pageRef = useRef(page)
  pageRef.current = page
  const onPageChangeRef = useRef(onPageChange)
  onPageChangeRef.current = onPageChange
  const pageResetPendingRef = useRef(false)

  useEffect(() => {
    const timerId = window.setTimeout(() => {
      if (debouncedRef.current === searchText) return
      const needPageReset = pageRef.current !== 1
      if (needPageReset) {
        pageResetPendingRef.current = true
      }
      setDebouncedSearchText(searchText)
      if (needPageReset) {
        onPageChangeRef.current?.(1)
      }
    }, SEARCH_DEBOUNCE_MS)
    return () => window.clearTimeout(timerId)
  }, [searchText])

  useEffect(() => {
    if (searchText !== debouncedSearchText) {
      return
    }
    if (pageResetPendingRef.current) {
      if (page !== 1) {
        return
      }
      pageResetPendingRef.current = false
    }

    let cancelled = false
    const fetchList = async () => {
      setIsLoading(true)
      try {
        const keyword = debouncedSearchText.trim() || null
        const parsedType = postcardType ? (postcardType as PostcardType) : null

        const result = await invoke<{ items: SendStatusItemDto[]; total: number }>(
          'search_send_status',
          {
            year,
            postcardType: parsedType,
            status,
            receiptYear,
            keyword,
            limit: pageSize,
            offset: (page - 1) * pageSize,
          },
        )

        const clamped = clampPage(page, result.total, pageSize)
        if (clamped !== page) {
          if (!cancelled) {
            setItems(result.items.map(fromSendStatusItemDto))
            setTotal(result.total)
            setError(null)
            onPageChange?.(clamped)
          }
          return
        }

        if (cancelled) return
        setItems(result.items.map(fromSendStatusItemDto))
        setTotal(result.total)
        setError(null)
      } catch (fetchError) {
        if (cancelled) return
        console.error('Failed to fetch send status list:', fetchError)
        setError(POSTCARD_SEND_OPERATION_ERROR_MESSAGE)
        setItems([])
        setTotal(0)
      } finally {
        if (!cancelled) {
          setIsLoading(false)
        }
      }
    }

    fetchList()

    return () => {
      cancelled = true
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- intentional
  }, [
    debouncedSearchText,
    searchText,
    year,
    postcardType,
    status,
    receiptYear,
    page,
    pageSize,
    reloadToken,
  ])

  const reload = () => {
    setReloadToken((prev) => prev + 1)
  }

  return {
    items,
    total,
    isLoading,
    error,
    settledSearchText: debouncedSearchText,
    reload,
  }
}
