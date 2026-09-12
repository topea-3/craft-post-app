import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { SENDER_OPERATION_ERROR_MESSAGE } from './messages'
import { fromSenderEntryDto, type SenderEntryDto, type SenderEntryListItem } from './types'

type UseSenderEntryListParams = {
  page: number
  pageSize: number
  /** false のとき API を呼ばない（ダイアログ未表示時など） */
  enabled?: boolean
}

type UseSenderEntryListResult = {
  items: SenderEntryListItem[]
  isLoading: boolean
  error: string | null
  hasNext: boolean
  reload: () => void
}

export function useSenderEntryList({
  page,
  pageSize,
  enabled = true,
}: UseSenderEntryListParams): UseSenderEntryListResult {
  const [items, setItems] = useState<SenderEntryListItem[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [hasNext, setHasNext] = useState(false)
  const [reloadToken, setReloadToken] = useState(0)

  useEffect(() => {
    if (!enabled) {
      return
    }

    let cancelled = false
    const fetchList = async () => {
      setIsLoading(true)
      try {
        const limit = pageSize + 1
        const offset = (page - 1) * pageSize
        const dtos = await invoke<SenderEntryDto[]>('list_sender_entries', {
          limit,
          offset,
        })
        if (cancelled) return
        setHasNext(dtos.length > pageSize)
        setItems(dtos.slice(0, pageSize).map(fromSenderEntryDto))
        setError(null)
      } catch (e) {
        if (cancelled) return
        console.error('Failed to fetch sender entry list:', e)
        setItems([])
        setHasNext(false)
        setError(SENDER_OPERATION_ERROR_MESSAGE)
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
  }, [page, pageSize, reloadToken, enabled])

  const reload = () => {
    setReloadToken((prev) => prev + 1)
  }

  return { items, isLoading, error, hasNext, reload }
}

