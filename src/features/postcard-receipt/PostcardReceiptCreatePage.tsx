import { useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import { PostcardReceiptForm } from './PostcardReceiptForm'
import { usePostcardReceiptForm } from './usePostcardReceiptForm'

export function PostcardReceiptCreatePage() {
  const navigate = useNavigate()

  const handleSuccess = useCallback(
    (id: string) => {
      navigate(`/receipts/${id}`)
    },
    [navigate],
  )

  const form = usePostcardReceiptForm(handleSuccess)

  const handleCancel = () => {
    if (form.isDirty) {
      const confirmed = window.confirm('入力内容を破棄して一覧に戻りますか？')
      if (!confirmed) return
    }
    navigate('/receipts')
  }

  return (
    <div className="address-form-container">
      <header className="address-form-header">
        <h1>受取履歴新規作成</h1>
        <button type="button" className="link-button" onClick={handleCancel}>
          キャンセル
        </button>
      </header>
      <PostcardReceiptForm form={form} onCancel={handleCancel} />
    </div>
  )
}
