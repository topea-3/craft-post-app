import { useEffect, useState } from 'react'
import { resolveSendYear } from '../api'
import type { PostcardType } from '../types'

export type SendYearDecisionView =
  | { kind: 'year'; year: number }
  | { kind: 'test_print' }

/** Local 今日基準の送付年決定（サーバ）を購読する */
export function useSendYearDecision(postcardType: PostcardType) {
  const [decision, setDecision] = useState<SendYearDecisionView | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    ;(async () => {
      try {
        const next = await resolveSendYear(postcardType)
        if (cancelled) return
        setDecision(next)
        setError(null)
      } catch (e) {
        if (cancelled) return
        console.error('resolve_send_year failed:', e)
        setDecision(null)
        setError(String(e))
      }
    })()
    return () => {
      cancelled = true
    }
  }, [postcardType])

  return { decision, error, isTestPrint: decision?.kind === 'test_print' }
}
