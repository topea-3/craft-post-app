import { PRINT_TEST_PRINT_INFO_MESSAGE } from '../messages'

type Props = {
  visible: boolean
}

/** テスト印刷期間の Info（印刷フロー各画面上部） */
export function PrintTestPrintBanner({ visible }: Props) {
  if (!visible) return null
  return (
    <p className="print-test-print-info" role="status">
      {PRINT_TEST_PRINT_INFO_MESSAGE}
    </p>
  )
}
