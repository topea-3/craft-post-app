import { useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { PostcardReceiptForm } from './PostcardReceiptForm'
import { usePostcardReceiptEditForm } from './usePostcardReceiptForm'
import type { PostcardReceiptDto } from './types'
import { formValuesFromDetail, fromPostcardReceiptDtoToDetail, createInitialPostcardReceiptFormValues } from './types'
import { POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE } from './messages'

export function PostcardReceiptEditPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [initialValues, setInitialValues] = useState<ReturnType<typeof formValuesFromDetail> | null>(
    null,
  )
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!id) {
      setError('ID が指定されていません。')
      return
    }

    let cancelled = false
    const fetchDetail = async () => {
      setIsLoading(true)
      try {
        const dto = await invoke<PostcardReceiptDto>('get_postcard_receipt', { id })
        if (cancelled) return
        setInitialValues(formValuesFromDetail(fromPostcardReceiptDtoToDetail(dto)))
        setError(null)
      } catch (fetchError) {
        if (cancelled) return
        console.error('Failed to fetch postcard receipt for edit:', fetchError)
        setError(POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE)
      } finally {
        if (!cancelled) setIsLoading(false)
      }
    }

    fetchDetail()
    return () => {
      cancelled = true
    }
  }, [id])

  const handleSuccess = useMemo(
    () => () => {
      navigate(`/receipts/${id}`, { state: { flash: '受取履歴を更新しました。' } })
    },
    [id, navigate],
  )

  const form = usePostcardReceiptEditForm(
    id ?? '',
    initialValues ?? createInitialPostcardReceiptFormValues(),
    handleSuccess,
  )

  const handleCancel = () => {
    if (form.isDirty) {
      const confirmed = window.confirm('編集中の内容を破棄して詳細に戻りますか？')
      if (!confirmed) return
    }
    navigate(`/receipts/${id}`)
  }

  if (isLoading || !initialValues) {
    return (
      <div className="address-form-container">
        <p>{isLoading ? '読み込み中です…' : error ?? '受取履歴が見つかりませんでした。'}</p>
      </div>
    )
  }

  return (
    <div className="address-form-container">
      <header className="address-form-header">
        <h1>受取履歴編集</h1>
        <button type="button" className="link-button" onClick={handleCancel}>
          キャンセル
        </button>
      </header>
      <PostcardReceiptForm form={form} onCancel={handleCancel} />
    </div>
  )
}
