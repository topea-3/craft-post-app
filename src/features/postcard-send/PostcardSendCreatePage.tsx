import { useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { PostcardSendForm } from './PostcardSendForm'
import { usePostcardSendForm } from './usePostcardSendForm'

export function PostcardSendCreatePage() {
  const navigate = useNavigate()

  const onSuccess = useCallback(
    (id: string) => {
      navigate(`/sends/${id}`)
    },
    [navigate],
  )

  const form = usePostcardSendForm({
    mode: 'create',
    onSuccess,
  })

  const handleCancel = () => {
    if (form.isDirty) {
      const confirmed = window.confirm('入力内容を破棄して一覧に戻りますか？')
      if (!confirmed) return
    }
    navigate('/sends')
  }

  return (
    <div className="address-form-container">
      <header className="address-form-header">
        <h1>送付履歴新規作成</h1>
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
