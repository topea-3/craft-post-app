import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { clampPage } from '../../lib/pagination'
import type { PostcardSendDto, PostcardSendListItem, PostcardSendSource, PostcardType } from './types'
import { POSTCARD_SEND_OPERATION_ERROR_MESSAGE } from './messages'
import { fromPostcardSendDto } from './types'

const SEARCH_DEBOUNCE_MS = 300

type UsePostcardSendListParams = {
  searchText: string
  year: string
  postcardType: string
  source: string
  page: number
  pageSize: number
  onPageChange?: (page: number) => void
}

type UsePostcardSendListResult = {
  items: PostcardSendListItem[]
  total: number
  isLoading: boolean
  error: string | null
  settledSearchText: string
  isDebouncing: boolean
  reload: () => void
}

async function searchPostcardSends(args: {
  keyword: string | null
  year: number | null
  postcardType: PostcardType | null
  source: PostcardSendSource | null
  limit: number
  offset: number
}): Promise<{ items: PostcardSendDto[]; total: number }> {
  return invoke<{ items: PostcardSendDto[]; total: number }>('search_postcard_sends', {
    keyword: args.keyword,
    year: args.year,
    postcardType: args.postcardType,
    addressEntryId: null,
    source: args.source,
    limit: args.limit,
    offset: args.offset,
    sortOrder: 'desc',
  })
}

export function usePostcardSendList(params: UsePostcardSendListParams): UsePostcardSendListResult {
  const { searchText, year, postcardType, source, page, pageSize, onPageChange } = params
  const [debouncedSearchText, setDebouncedSearchText] = useState(searchText)
  const [items, setItems] = useState<PostcardSendListItem[]>([])
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
        const parsedYear = year ? Number(year) : null
        const parsedType = postcardType ? (postcardType as PostcardType) : null
        const parsedSource = source ? (source as PostcardSendSource) : null

        const result = await searchPostcardSends({
          keyword,
          year: parsedYear,
          postcardType: parsedType,
          source: parsedSource,
          limit: pageSize,
          offset: (page - 1) * pageSize,
        })

        const clamped = clampPage(page, result.total, pageSize)
        if (clamped !== page) {
          if (!cancelled) {
            setItems(result.items.map(fromPostcardSendDto))
            setTotal(result.total)
            setError(null)
            onPageChange?.(clamped)
          }
          return
        }

        if (cancelled) return
        setItems(result.items.map(fromPostcardSendDto))
        setTotal(result.total)
        setError(null)
      } catch (fetchError) {
        if (cancelled) return
        console.error('Failed to fetch postcard send list:', fetchError)
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
  }, [debouncedSearchText, searchText, year, postcardType, source, page, pageSize, reloadToken])

  const reload = () => {
    setReloadToken((prev) => prev + 1)
  }

  return {
    items,
    total,
    isLoading,
    error,
    settledSearchText: debouncedSearchText,
    isDebouncing: searchText !== debouncedSearchText,
    reload,
  }
}
