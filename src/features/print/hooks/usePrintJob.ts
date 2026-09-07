import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { PostcardLayoutSpec } from '../layout/layoutTypes'
import { clampOffset } from '../layout/layoutMath'
import { nengaLayoutSpec } from '../layout/nengaLayoutSpec'
import { mochuLayoutSpec } from '../layout/mochuLayoutSpec'
import type { PrintLayerId } from '../layout/printLayers'
import { ALL_PRINT_LAYER_IDS } from '../layout/printLayers'
import { listPrintLayoutPreferences, savePrintLayoutPreferences } from '../api'
import type { LayoutOffsets, PostcardType, PrintJobItem } from '../types'
import { createDefaultLayoutOffsets, createDefaultLayerVisibility } from '../types'
import { PRINT_OPERATION_ERROR_MESSAGE } from '../messages'

function layoutSpecFor(type: PostcardType): PostcardLayoutSpec {
  return type === 'mochu' ? mochuLayoutSpec : nengaLayoutSpec
}

function offsetsEqual(a: LayoutOffsets, b: LayoutOffsets): boolean {
  for (const id of ALL_PRINT_LAYER_IDS) {
    if (a[id].dx !== b[id].dx || a[id].dy !== b[id].dy) return false
  }
  return true
}

export function usePrintJob(params: {
  postcardType: PostcardType
  items: PrintJobItem[]
  onItemsChange: (items: PrintJobItem[]) => void
}) {
  const { postcardType, items, onItemsChange } = params
  const layoutSpec = useMemo(() => layoutSpecFor(postcardType), [postcardType])

  const [layoutOffsets, setLayoutOffsets] = useState<LayoutOffsets>(() =>
    createDefaultLayoutOffsets(),
  )
  const [savedOffsets, setSavedOffsets] = useState<LayoutOffsets>(() =>
    createDefaultLayoutOffsets(),
  )
  const [prefsLoading, setPrefsLoading] = useState(true)
  const [appliedType, setAppliedType] = useState<PostcardType | null>(null)
  const [prefsError, setPrefsError] = useState<string | null>(null)
  const [savingPrefs, setSavingPrefs] = useState(false)
  const [selectedLayerId, setSelectedLayerId] = useState<PrintLayerId | null>(
    'recipient.primaryLast',
  )
  const loadTokenRef = useRef(0)
  const dirtyRef = useRef(false)

  const isDirty = !offsetsEqual(layoutOffsets, savedOffsets)
  dirtyRef.current = isDirty
  const prefsReady = appliedType === postcardType && !prefsLoading

  useEffect(() => {
    const token = ++loadTokenRef.current
    let cancelled = false
    ;(async () => {
      setPrefsLoading(true)
      setPrefsError(null)
      try {
        const prefs = await listPrintLayoutPreferences(postcardType)
        if (cancelled || token !== loadTokenRef.current) return
        const next = createDefaultLayoutOffsets()
        for (const p of prefs) {
          next[p.layerId] = {
            dx: p.offsetXPt,
            dy: p.offsetYPt,
          }
        }
        for (const id of ALL_PRINT_LAYER_IDS) {
          const origin = layoutSpecFor(postcardType).layers[id].originMm
          next[id] = clampOffset(origin, next[id], layoutSpecFor(postcardType).printable)
        }
        // ロード中にユーザが編集済みなら上書きしない（通常は prefsReady=false で編集不可）
        if (!dirtyRef.current) {
          setLayoutOffsets(next)
        }
        setSavedOffsets(next)
        setAppliedType(postcardType)
      } catch (e) {
        console.error('Failed to load print layout preferences:', e)
        if (!cancelled && token === loadTokenRef.current) {
          setPrefsError(PRINT_OPERATION_ERROR_MESSAGE)
          const defaults = createDefaultLayoutOffsets()
          if (!dirtyRef.current) {
            setLayoutOffsets(defaults)
          }
          setSavedOffsets(defaults)
          setAppliedType(postcardType)
        }
      } finally {
        if (!cancelled && token === loadTokenRef.current) {
          setPrefsLoading(false)
        }
      }
    })()
    return () => {
      cancelled = true
    }
  }, [postcardType])

  const setOffset = useCallback(
    (layerId: PrintLayerId, dx: number, dy: number) => {
      if (appliedType !== postcardType) return
      setLayoutOffsets((prev) => {
        const origin = layoutSpec.layers[layerId].originMm
        const clamped = clampOffset(origin, { dx, dy }, layoutSpec.printable)
        return { ...prev, [layerId]: clamped }
      })
    },
    [appliedType, layoutSpec, postcardType],
  )

  const resetOffsets = useCallback((layerId?: PrintLayerId) => {
    setLayoutOffsets((prev) => {
      if (layerId) {
        return { ...prev, [layerId]: { dx: 0, dy: 0 } }
      }
      return createDefaultLayoutOffsets()
    })
  }, [])

  const setLayerVisibility = useCallback(
    (itemIndex: number, layerId: PrintLayerId, visible: boolean) => {
      const next = items.map((item, i) => {
        if (i !== itemIndex) return item
        return {
          ...item,
          layerVisibility: {
            ...item.layerVisibility,
            [layerId]: visible,
          },
        }
      })
      onItemsChange(next)
    },
    [items, onItemsChange],
  )

  const ensureItemVisibility = useCallback(
    (list: PrintJobItem[]): PrintJobItem[] =>
      list.map((item) => ({
        ...item,
        layerVisibility: {
          ...createDefaultLayerVisibility(),
          ...item.layerVisibility,
        },
      })),
    [],
  )

  const savePrefs = useCallback(async () => {
    if (appliedType !== postcardType || prefsLoading) return false
    setSavingPrefs(true)
    try {
      const clamped = createDefaultLayoutOffsets()
      for (const id of ALL_PRINT_LAYER_IDS) {
        clamped[id] = clampOffset(
          layoutSpec.layers[id].originMm,
          layoutOffsets[id],
          layoutSpec.printable,
        )
      }
      await savePrintLayoutPreferences(postcardType, clamped)
      setLayoutOffsets(clamped)
      setSavedOffsets(clamped)
      setPrefsError(null)
      return true
    } catch (e) {
      console.error('Failed to save print layout preferences:', e)
      setPrefsError(PRINT_OPERATION_ERROR_MESSAGE)
      return false
    } finally {
      setSavingPrefs(false)
    }
  }, [appliedType, layoutOffsets, layoutSpec, postcardType, prefsLoading])

  const discardDirtyOffsets = useCallback(() => {
    setLayoutOffsets(savedOffsets)
  }, [savedOffsets])

  return {
    layoutSpec,
    layoutOffsets,
    isDirty,
    prefsLoading,
    prefsReady,
    prefsError,
    savingPrefs,
    selectedLayerId,
    setSelectedLayerId,
    setOffset,
    resetOffsets,
    setLayerVisibility,
    ensureItemVisibility,
    savePrefs,
    discardDirtyOffsets,
  }
}
