import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
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
}

type UsePostcardReceiptListResult = {
  items: PostcardReceiptListItem[]
  total: number
  isLoading: boolean
  error: string | null
  reload: () => void
}

export function usePostcardReceiptList(
  params: UsePostcardReceiptListParams,
): UsePostcardReceiptListResult {
  const { searchText, year, category, addressEntryId, page, pageSize } = params
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
        const offset = (page - 1) * pageSize

        const result = await invoke<{ items: PostcardReceiptDto[]; total: number }>(
          'search_postcard_receipts',
          {
            keyword,
            year: parsedYear,
            category: parsedCategory,
            addressEntryId,
            includeDeleted: false,
            limit,
            offset,
            sortOrder: 'desc',
          },
        )

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
  }, [debouncedSearchText, year, category, addressEntryId, page, pageSize, reloadToken])

  const reload = () => {
    setReloadToken((prev) => prev + 1)
  }

  return { items, total, isLoading, error, reload }
}
