import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate, useParams } from 'react-router-dom'
import { POSTCARD_SEND_OPERATION_ERROR_MESSAGE } from './messages'
import { PostcardSendForm } from './PostcardSendForm'
import type { PostcardSendDto, PostcardSendFormValues } from './types'
import { formValuesFromDetail, fromPostcardSendDtoToDetail } from './types'
import { usePostcardSendForm } from './usePostcardSendForm'

type LoadedProps = {
  id: string
  initialValues: PostcardSendFormValues
  expectedUpdatedAt: string
  baselineSentOn: string
}

function PostcardSendEditLoaded({
  id,
  initialValues,
  expectedUpdatedAt,
  baselineSentOn,
}: LoadedProps) {
  const navigate = useNavigate()

  const onSuccess = useMemo(
    () => () => {
      navigate(`/sends/${id}`)
    },
    [id, navigate],
  )

  // initialValues は親がロード完了後に一度だけ渡す（参照変化で sync effect を回さない）
  const form = usePostcardSendForm({
    mode: 'edit',
    sendId: id,
    expectedUpdatedAt,
    initialValues,
    baselineSentOn,
    onSuccess,
  })

  const handleCancel = () => {
    if (form.isDirty) {
      const confirmed = window.confirm('入力内容を破棄して詳細に戻りますか？')
      if (!confirmed) return
    }
    navigate(`/sends/${id}`)
  }

  return (
    <div className="address-form-container">
      <header className="address-form-header">
        <h1>送付履歴編集</h1>
        <button
          type="button"
          className="link-button"
          onClick={handleCancel}
          disabled={form.isSubmitting}
        >
          キャンセル
        </button>
      </header>
      <PostcardSendForm form={form} onCancel={handleCancel} />
    </div>
  )
}

export function PostcardSendEditPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [initialValues, setInitialValues] = useState<PostcardSendFormValues | null>(null)
  const [expectedUpdatedAt, setExpectedUpdatedAt] = useState<string | null>(null)
  const [baselineSentOn, setBaselineSentOn] = useState<string | null>(null)
  const [loadError, setLoadError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (!id) return
    let cancelled = false
    ;(async () => {
      setLoading(true)
      try {
        const dto = await invoke<PostcardSendDto>('get_postcard_send', { id })
        if (cancelled) return
        const detail = fromPostcardSendDtoToDetail(dto)
        setInitialValues(formValuesFromDetail(detail))
        setExpectedUpdatedAt(detail.updatedAt)
        setBaselineSentOn(detail.sentOn)
        setLoadError(null)
      } catch (e) {
        console.error(e)
        if (!cancelled) {
          setLoadError(POSTCARD_SEND_OPERATION_ERROR_MESSAGE)
          setInitialValues(null)
          setExpectedUpdatedAt(null)
          setBaselineSentOn(null)
        }
      } finally {
        if (!cancelled) setLoading(false)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [id])

  if (loading) {
    return <p>読み込み中…</p>
  }
  if (loadError || !initialValues || !expectedUpdatedAt || !baselineSentOn || !id) {
    return (
      <div className="address-form-container">
        <p className="address-form-error">{loadError ?? '送付履歴が見つかりません。'}</p>
        <button type="button" className="link-button" onClick={() => navigate('/sends')}>
          一覧へ
        </button>
      </div>
    )
  }

  return (
    <PostcardSendEditLoaded
      id={id}
      initialValues={initialValues}
      expectedUpdatedAt={expectedUpdatedAt}
      baselineSentOn={baselineSentOn}
    />
  )
}
