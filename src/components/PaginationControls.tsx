type PaginationControlsProps = {
  currentPage: number
  onPrev: () => void
  onNext: () => void
  /** totalPages が未指定のとき、次ページの有無で next を制御する */
  hasNext?: boolean
  totalPages?: number
  className?: string
}

export function PaginationControls({
  currentPage,
  onPrev,
  onNext,
  hasNext = false,
  totalPages,
  className,
}: PaginationControlsProps) {
  const canGoPrev = currentPage > 1
  const canGoNext =
    totalPages !== undefined ? currentPage < totalPages : hasNext
  const pageLabel =
    totalPages !== undefined ? `${currentPage} / ${totalPages}` : `${currentPage}`

  return (
    <div className={className ?? 'pagination-controls'}>
      <button type="button" onClick={onPrev} disabled={!canGoPrev}>
        前へ
      </button>
      <span className="pagination-controls-page-info">{pageLabel}</span>
      <button type="button" onClick={onNext} disabled={!canGoNext}>
        次へ
      </button>
    </div>
  )
}
