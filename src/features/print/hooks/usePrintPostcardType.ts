import { useCallback, useEffect, useState } from 'react'
import type { PostcardType } from '../types'

const STORAGE_KEY = 'printPostcardType'

function readType(): PostcardType {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY)
    if (raw === 'nenga' || raw === 'mochu') return raw
  } catch {
    /* ignore */
  }
  return 'nenga'
}

export function usePrintPostcardType() {
  const [postcardType, setPostcardTypeState] = useState<PostcardType>(() => readType())

  useEffect(() => {
    sessionStorage.setItem(STORAGE_KEY, postcardType)
  }, [postcardType])

  const setPostcardType = useCallback((type: PostcardType) => {
    setPostcardTypeState(type)
  }, [])

  return { postcardType, setPostcardType }
}
