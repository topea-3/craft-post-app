import { useNavigate } from 'react-router-dom'
import { usePrintPostcardType } from '../hooks/usePrintPostcardType'
import { useSendYearDecision } from '../hooks/useSendYearDecision'
import { PrintTestPrintBanner } from '../components/PrintTestPrintBanner'
import { PRINT_OPERATION_ERROR_MESSAGE } from '../messages'
import { POSTCARD_TYPE_OPTIONS, type PostcardType } from '../types'

export function PrintTypePage() {
  const navigate = useNavigate()
  const { postcardType, setPostcardType } = usePrintPostcardType()
  const { isTestPrint, error: sendYearError } = useSendYearDecision(postcardType)

  return (
    <div className="page">
      <header className="page-header">
        <h1 className="page-title">印刷するはがきの種類</h1>
      </header>

      {sendYearError ? (
        <p className="form-error" role="alert">
          {PRINT_OPERATION_ERROR_MESSAGE}
        </p>
      ) : null}
      <PrintTestPrintBanner visible={isTestPrint} />

      <div className="form-field">
        <label htmlFor="print-postcard-type">
          種別 <span aria-hidden="true">*</span>
        </label>
        <select
          id="print-postcard-type"
          value={postcardType}
          onChange={(e) => setPostcardType(e.target.value as PostcardType)}
        >
          {POSTCARD_TYPE_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      <div className="form-actions">
        <button type="button" className="secondary" onClick={() => navigate('/addresses')}>
          キャンセル
        </button>
        <button type="button" onClick={() => navigate('/print/select')}>
          宛名選択画面へ
        </button>
      </div>
    </div>
  )
}
