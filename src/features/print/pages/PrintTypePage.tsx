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
    <div className="print-page">
      <div className="print-page-header">
        <h1 className="print-page-title">印刷するはがきの種類</h1>
      </div>

      {sendYearError ? (
        <p className="print-error" role="alert">
          {PRINT_OPERATION_ERROR_MESSAGE}
        </p>
      ) : null}
      <PrintTestPrintBanner visible={isTestPrint} />

      <section className="print-type-panel" aria-labelledby="print-type-panel-title">
        <h2 id="print-type-panel-title" className="print-type-panel-title">
          はがき種別
        </h2>
        <label className="print-type-field" htmlFor="print-postcard-type">
          <span>
            種別 <span aria-hidden="true">*</span>
          </span>
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
        </label>
      </section>

      <div className="print-page-footer print-type-footer">
        <div className="print-page-actions">
          <button
            type="button"
            className="btn btn-label btn-normal"
            onClick={() => navigate('/addresses')}
          >
            キャンセル
          </button>
          <button
            type="button"
            className="btn btn-label btn-primary print-primary-button"
            onClick={() => navigate('/print/select')}
          >
            宛名選択画面へ →
          </button>
        </div>
      </div>
    </div>
  )
}
