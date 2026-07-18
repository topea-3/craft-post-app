import { useEffect, useState } from 'react'
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

  useEffect(() => {
    const id = window.setTimeout(() => {
      setDebouncedSearchText(searchText)
    }, SEARCH_DEBOUNCE_MS)
    return () => window.clearTimeout(id)
  }, [searchText])

  useEffect(() => {
    let cancelled = false
    const fetchList = async () => {
      setIsLoading(true)
      try {
        const keyword = debouncedSearchText.trim() || null
        const parsedYear = year ? Number(year) : null
        const parsedCategory = category ? (category as PostcardReceiptCategory) : null
        const limit = pageSize

        let requestPage = page
        let result = await searchPostcardReceipts({
          keyword,
          year: parsedYear,
          category: parsedCategory,
          addressEntryId,
          limit,
          offset: (requestPage - 1) * pageSize,
        })

        // 削除などで page が総ページを超えた場合は有効ページへ寄せて再取得する
        const clamped = clampPage(requestPage, result.total, pageSize)
        if (clamped !== requestPage) {
          requestPage = clamped
          result = await searchPostcardReceipts({
            keyword,
            year: parsedYear,
            category: parsedCategory,
            addressEntryId,
            limit,
            offset: (requestPage - 1) * pageSize,
          })
          if (!cancelled) {
            onPageChange?.(requestPage)
          }
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
  }, [debouncedSearchText, year, category, addressEntryId, page, pageSize, reloadToken])

  const reload = () => {
    setReloadToken((prev) => prev + 1)
  }

  return { items, total, isLoading, error, reload }
}
