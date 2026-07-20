import { useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { PostcardReceiptForm } from './PostcardReceiptForm'
import { usePostcardReceiptEditForm } from './usePostcardReceiptForm'
import type { PostcardReceiptDto, PostcardReceiptFormValues } from './types'
import { formValuesFromDetail, fromPostcardReceiptDtoToDetail } from './types'
import { POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE } from './messages'

type LoadedProps = {
  id: string
  initialValues: PostcardReceiptFormValues
  expectedUpdatedAt: string
}

function PostcardReceiptEditFormLoaded({ id, initialValues, expectedUpdatedAt }: LoadedProps) {
  const navigate = useNavigate()

  const handleSuccess = useMemo(
    () => () => {
      navigate(`/receipts/${id}`, { state: { flash: '受取履歴を更新しました。' } })
    },
    [id, navigate],
  )

  // initialValues は親がロード完了後に一度だけ渡す
  const form = usePostcardReceiptEditForm(id, initialValues, expectedUpdatedAt, handleSuccess)

  const handleCancel = () => {
    if (form.isDirty) {
      const confirmed = window.confirm('編集中の内容を破棄して詳細に戻りますか？')
      if (!confirmed) return
    }
    navigate(`/receipts/${id}`)
  }

  return (
    <div className="address-form-container">
      <header className="address-form-header">
        <h1>受取履歴編集</h1>
        <button
          type="button"
          className="link-button"
          onClick={handleCancel}
          disabled={form.isSubmitting}
        >
          キャンセル
        </button>
      </header>
      <PostcardReceiptForm form={form} onCancel={handleCancel} />
    </div>
  )
}

export function PostcardReceiptEditPage() {
  const { id } = useParams<{ id: string }>()
  const [initialValues, setInitialValues] = useState<PostcardReceiptFormValues | null>(null)
  const [expectedUpdatedAt, setExpectedUpdatedAt] = useState<string | null>(null)
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
        setExpectedUpdatedAt(dto.updated_at)
        setError(null)
      } catch (fetchError) {
        if (cancelled) return
        console.error('Failed to fetch postcard receipt for edit:', fetchError)
        setError(POSTCARD_RECEIPT_OPERATION_ERROR_MESSAGE)
        setInitialValues(null)
        setExpectedUpdatedAt(null)
      } finally {
        if (!cancelled) setIsLoading(false)
      }
    }

    fetchDetail()
    return () => {
      cancelled = true
    }
  }, [id])

  if (isLoading) {
    return (
      <div className="address-form-container">
        <p>読み込み中です…</p>
      </div>
    )
  }

  if (!id || error || !initialValues || !expectedUpdatedAt) {
    return (
      <div className="address-form-container">
        <p>{error ?? '受取履歴が見つかりませんでした。'}</p>
      </div>
    )
  }

  // ロード成功後にだけフォーム hook をマウントし、プレースホルダ初期値による同期ループを避ける
  return (
    <PostcardReceiptEditFormLoaded
      id={id}
      initialValues={initialValues}
      expectedUpdatedAt={expectedUpdatedAt}
    />
  )
}
