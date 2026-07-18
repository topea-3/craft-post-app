import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { clampPage } from '../../lib/pagination'
import type { PostcardReceiptCategory, PostcardReceiptDto, PostcardReceiptListItem } from './types'
import { POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE } from './messages'
import { fromPostcardReceiptDto } from './types'

const SEARCH_DEBOUNCE_MS = 300

type UsePostcardReceiptListParams = {
  searchText: string
  year: string
  category: string
  addressEntryId: string | null
  page: number
  pageSize: number
  /** page が総ページ数を超えたとき、補正後のページを親へ返す */
  onPageChange?: (page: number) => void
}

type UsePostcardReceiptListResult = {
  items: PostcardReceiptListItem[]
  total: number
  isLoading: boolean
  error: string | null
  reload: () => void
}

async function searchPostcardReceipts(args: {
  keyword: string | null
  year: number | null
  category: PostcardReceiptCategory | null
  addressEntryId: string | null
  limit: number
  offset: number
}): Promise<{ items: PostcardReceiptDto[]; total: number }> {
  return invoke<{ items: PostcardReceiptDto[]; total: number }>('search_postcard_receipts', {
    keyword: args.keyword,
    year: args.year,
    category: args.category,
    addressEntryId: args.addressEntryId,
    includeDeleted: false,
    limit: args.limit,
    offset: args.offset,
    sortOrder: 'desc',
  })
}

export function usePostcardReceiptList(
  params: UsePostcardReceiptListParams,
): UsePostcardReceiptListResult {
  const { searchText, year, category, addressEntryId, page, pageSize, onPageChange } = params
  const [debouncedSearchText, setDebouncedSearchText] = useState(searchText)
  const [items, setItems] = useState<PostcardReceiptListItem[]>([])
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

  // 検索確定とページリセットを同じ debounce 遷移にまとめ、未確定入力では取得しない
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
    // 入力確定前は旧条件での再取得を起こさない
    if (searchText !== debouncedSearchText) {
      return
    }
    // 検索確定に伴う page リセット待ち（旧 page での余分な取得を避ける）
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
        const parsedYear = year ? Number(year) : null
        const parsedCategory = category ? (category as PostcardReceiptCategory) : null
        const limit = pageSize

        const result = await searchPostcardReceipts({
          keyword,
          year: parsedYear,
          category: parsedCategory,
          addressEntryId,
          limit,
          offset: (page - 1) * pageSize,
        })

        // 削除などで page が総ページを超えた場合は親へ補正を返し、再 fetch は effect 再実行に任せる
        const clamped = clampPage(page, result.total, pageSize)
        if (clamped !== page) {
          if (!cancelled) {
            setTotal(result.total)
            setError(null)
            onPageChange?.(clamped)
          }
          return
        }

        if (cancelled) return
        setItems(result.items.map(fromPostcardReceiptDto))
        setTotal(result.total)
        setError(null)
      } catch (fetchError) {
        if (cancelled) return
        console.error('Failed to fetch postcard receipt list:', fetchError)
        setError(POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE)
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
    // onPageChange は親の setPage 想定のため deps に含めない（参照変化での再 fetch を避ける）
    // eslint-disable-next-line react-hooks/exhaustive-deps -- intentional
  }, [debouncedSearchText, searchText, year, category, addressEntryId, page, pageSize, reloadToken])

  const reload = () => {
    setReloadToken((prev) => prev + 1)
  }

  return { items, total, isLoading, error, reload }
}
