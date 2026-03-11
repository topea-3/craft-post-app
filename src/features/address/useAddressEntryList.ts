import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { AddressEntryListItem, AddressEntryDto } from './types'
import { fromAddressEntryDto } from './types'

export type ListSortKey = 'nameKana' | 'updatedAt'
export type ListSortOrder = 'asc' | 'desc'

type UseAddressEntryListParams = {
  searchText: string
  sortKey: ListSortKey
  sortOrder: ListSortOrder
}

type UseAddressEntryListResult = {
  items: AddressEntryListItem[]
  isLoading: boolean
  error: string | null
  reload: () => void
}

export function useAddressEntryList(
  params: UseAddressEntryListParams,
): UseAddressEntryListResult {
  const { searchText, sortKey, sortOrder } = params
  const [items, setItems] = useState<AddressEntryListItem[]>([])
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

        const dtos = await invoke<AddressEntryDto[]>('search_address_entries', {
          keyword,
          sortKey: sortKeyArg,
          sortOrder: sortOrderArg,
          includeArchived: false,
          limit: null,
          offset: null,
        })

        if (cancelled) return
        setItems(dtos.map(fromAddressEntryDto))
        setError(null)
      } catch (e) {
        if (cancelled) return
        setError(String(e))
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
  }, [searchText, sortKey, sortOrder, reloadToken])

  const reload = () => {
    setReloadToken((prev) => prev + 1)
  }

  return { items, isLoading, error, reload }
}

