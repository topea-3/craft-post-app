import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { AddressEntryListItem, AddressEntryDto } from './types'
import { ADDRESS_OPERATION_ERROR_MESSAGE } from './messages'
import { fromAddressEntryDto } from './types'

export type ListSortKey = 'nameKana' | 'updatedAt'
export type ListSortOrder = 'asc' | 'desc'

type UseAddressEntryListParams = {
  searchText: string
  sortKey: ListSortKey
  sortOrder: ListSortOrder
  page: number
  pageSize: number
}

type UseAddressEntryListResult = {
  items: AddressEntryListItem[]
  total: number
  isLoading: boolean
  error: string | null
  reload: () => void
}

export function useAddressEntryList(
  params: UseAddressEntryListParams,
): UseAddressEntryListResult {
  const { searchText, sortKey, sortOrder, page, pageSize } = params
  const [items, setItems] = useState<AddressEntryListItem[]>([])
  const [total, setTotal] = useState(0)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [reloadToken, setReloadToken] = useState(0)

  useEffect(() => {
    let cancelled = false
    const fetchList = async () => {
      setIsLoading(true)
      try {
        const keyword = searchText.trim() || null
        const sortKeyArg = sortKey === 'updatedAt' ? 'updated_at' : 'name_kana'
        const sortOrderArg = sortOrder === 'desc' ? 'desc' : 'asc'
        const limit = pageSize
        const offset = (page - 1) * pageSize

        const result = await invoke<{ items: AddressEntryDto[]; total: number }>(
          'search_address_entries',
          {
            keyword,
            sortKey: sortKeyArg,
            sortOrder: sortOrderArg,
            includeArchived: false,
            limit,
            offset,
          },
        )

        if (cancelled) return
        setItems(result.items.map(fromAddressEntryDto))
        setTotal(result.total)
        setError(null)
      } catch (e) {
        if (cancelled) return
        setError(ADDRESS_OPERATION_ERROR_MESSAGE)
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
  }, [searchText, sortKey, sortOrder, page, pageSize, reloadToken])

  const reload = () => {
    setReloadToken((prev) => prev + 1)
  }

  return { items, total, isLoading, error, reload }
}

